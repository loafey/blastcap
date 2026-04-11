use sharpify::sharpify_types;
#[sharpify_types]
mod inner {
    use rkyv::Archive;
    use serde::{Deserialize, Serialize};
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
}
pub use inner::*;
