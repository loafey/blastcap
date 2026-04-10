use rapidhash::{RapidHashMap, quality::SeedableState};
use rkyv::Archive;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    fmt, fs,
    hash::{BuildHasher, Hash},
    io,
    marker::PhantomData,
    ops::Deref,
    path::{Path, PathBuf},
    sync::{LazyLock, OnceLock},
};
use walkdir::WalkDir;

static HASHER: LazyLock<SeedableState> = LazyLock::new(|| {
    SeedableState::custom(
        0xdeadbeef,
        &[
            0x2998b89d5cf7ebcf,
            0x5739954b350f0cb6,
            0x309f341055e40ff3,
            0x6e4f7e68f44e32e0,
            0x5f69b89319603cc2,
            0xc9671393592db796,
            0x9dd7847eb38817ba,
        ],
    )
});
fn hash<T: Hash>(val: T) -> u64 {
    HASHER.hash_one(val)
}

pub fn load<P: AsRef<Path>>(path: P) -> io::Result<DataSetInfo<Loaded>> {
    let mut info = path.as_ref().to_path_buf();
    info.push("Info.toml");
    let info = fs::read_to_string(info).unwrap();
    let data = toml::from_str::<DataSetInfo<NotLoaded>>(&info).unwrap();
    data.init(path.as_ref().to_path_buf())
}

trait InitState {}
#[derive(serde::Deserialize, Debug)]
pub struct NotLoaded;
impl InitState for NotLoaded {}
#[derive(serde::Deserialize, Debug)]
pub struct Loaded;
impl InitState for Loaded {}

trait Initialize {
    type Output;
    fn init(self, parent: PathBuf) -> io::Result<Self::Output>;
}

#[derive(Debug, Deserialize)]
#[allow(private_bounds)]
pub struct DataSetInfo<S: InitState = Loaded> {
    #[serde(skip_deserializing)]
    state: PhantomData<S>,
    pub name: String,
    pub version: String,
    pub cards: Directory<Card, S>,
}
impl Initialize for DataSetInfo<NotLoaded> {
    type Output = DataSetInfo<Loaded>;

    fn init(self, parent: PathBuf) -> io::Result<Self::Output> {
        let DataSetInfo {
            state: _,
            name,
            version,
            cards,
        } = self;
        Ok(DataSetInfo {
            state: PhantomData,
            name,
            version,
            cards: cards.init(parent)?,
        })
    }
}

#[derive(Deserialize)]
#[allow(private_bounds)]
pub struct Directory<T, S: InitState = Loaded> {
    #[serde(skip_deserializing)]
    state: PhantomData<S>,
    path: PathBuf,
    #[serde(skip_deserializing, default = "OnceLock::new")]
    loaded: OnceLock<RapidHashMap<u64, T>>,
}
impl<T> Deref for Directory<T, Loaded> {
    type Target = RapidHashMap<u64, T>;

    fn deref(&self) -> &Self::Target {
        self.loaded.get().unwrap()
    }
}
impl<T: fmt::Debug + DeserializeOwned, S: InitState> fmt::Debug for Directory<T, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.loaded.get() {
            Some(o) => o.fmt(f),
            None => unreachable!(),
        }
    }
}
impl<T: DeserializeOwned> Initialize for Directory<T, NotLoaded> {
    type Output = Directory<T, Loaded>;
    fn init(self, mut parent: PathBuf) -> io::Result<Self::Output> {
        let mut data = RapidHashMap::default();
        parent.push(&self.path);
        for path in WalkDir::new(parent) {
            let path = path?;
            let path = path.path();
            if path.is_dir() {
                continue;
            }
            if path.extension().map(|s| s != "toml").unwrap_or_default() {
                continue;
            }
            let string = fs::read_to_string(path)?;
            let content = toml::from_str(&string).map_err(|e| io::Error::other(format!("{e}")))?;

            if data.insert(hash(string), content).is_some() {
                panic!("hash collision; please change hasher")
            }
        }
        _ = self.loaded.set(data);
        Ok(Directory {
            state: PhantomData,
            path: self.path,
            loaded: self.loaded,
        })
    }
}

#[derive(
    Debug, Deserialize, Serialize, Clone, PartialEq, Archive, rkyv::Deserialize, rkyv::Serialize,
)]
pub enum CardType {
    Projectile,
}

#[derive(
    Debug, Deserialize, Serialize, Clone, PartialEq, Archive, rkyv::Deserialize, rkyv::Serialize,
)]
pub struct Card {
    pub name: String,
    pub projectile_speed: Option<f32>,
    pub r#type: CardType,
}
