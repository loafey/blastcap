use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use serde::Deserialize;

static ABILITY_MAP: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
    HashMap::from_iter(
        [
            ("Walk", "Walk dummy!"),
            ("Jump", "Jump dummy!"),
            ("Punch", "Punch a target!"),
        ]
        .into_iter()
        .map(|(s1, s2)| (s1.to_string(), s2.to_string())),
    )
});

#[derive(Debug, Deserialize, Clone)]
pub struct Abilities(HashSet<String>);
impl Abilities {
    pub fn get_map() -> &'static HashMap<String, String> {
        &ABILITY_MAP
    }

    pub fn get_keys(&self) -> Vec<String> {
        self.0.iter().cloned().collect()
    }

    pub fn contains(&self, ability: &str) -> bool {
        self.0.contains(ability)
    }
}
impl Default for Abilities {
    fn default() -> Self {
        let set = HashSet::from_iter(["Walk", "Jump", "Punch"].into_iter().map(|s| s.to_string()));
        Self(set)
    }
}
