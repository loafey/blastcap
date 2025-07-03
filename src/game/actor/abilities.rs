use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

static ABILITY_MAP: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    HashMap::from_iter(
        [("Walk", "Walk dummy!"), ("Jump", "Jump dummy!")]
            .into_iter()
            .map(|(s1, s2)| (s1.to_string(), s2.to_string())),
    )
});

#[derive(Debug)]
pub struct Abilities(HashSet<String>);
impl Abilities {
    pub fn get_map() -> &'static HashMap<String, String> {
        &ABILITY_MAP
    }

    pub fn get_keys(&self) -> Vec<String> {
        self.0.iter().cloned().collect()
    }
}
impl Default for Abilities {
    fn default() -> Self {
        let set = HashSet::from_iter(["Walk", "Jump"].into_iter().map(|s| s.to_string()));
        Self(set)
    }
}
