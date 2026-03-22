use serde::{Deserialize, de::DeserializeOwned};
use std::{
    fmt, fs, io,
    marker::PhantomData,
    ops::Deref,
    path::{Path, PathBuf},
    sync::OnceLock,
};
use walkdir::WalkDir;

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
    loaded: OnceLock<Vec<T>>,
}
impl<T> Deref for Directory<T, Loaded> {
    type Target = Vec<T>;

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
        let mut data = Vec::new();
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
            data.push(toml::from_str(&string).map_err(|e| io::Error::other(format!("{e}")))?);
        }
        _ = self.loaded.set(data);
        Ok(Directory {
            state: PhantomData,
            path: self.path,
            loaded: self.loaded,
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Card {
    pub name: String,
}
