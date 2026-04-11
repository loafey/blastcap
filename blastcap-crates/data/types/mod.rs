use sharpify::sharpify_types;
#[sharpify_types]
mod inner {
    use rkyv::Archive;
    use serde::{Deserialize, Serialize};
    #[derive(
        Debug, Deserialize, Serialize, Clone, PartialEq, Archive, rkyv::Deserialize, rkyv::Serialize,
    )]
    pub enum AttackType {
        Projectile,
    }

    #[derive(
        Debug, Deserialize, Serialize, Clone, PartialEq, Archive, rkyv::Deserialize, rkyv::Serialize,
    )]
    pub struct AttackData {
        pub projectile_speed: Option<f32>,
        pub r#type: AttackType,
    }

    #[derive(
        Debug, Deserialize, Serialize, Clone, PartialEq, Archive, rkyv::Deserialize, rkyv::Serialize,
    )]
    pub enum MovementType {
        Walk,
        Fly,
        Jump,
    }

    #[derive(
        Debug, Deserialize, Serialize, Clone, PartialEq, Archive, rkyv::Deserialize, rkyv::Serialize,
    )]
    pub struct MovementData {
        pub move_to_target: bool,
        pub r#type: MovementType,
    }

    #[derive(
        Debug, Deserialize, Serialize, Clone, PartialEq, Archive, rkyv::Deserialize, rkyv::Serialize,
    )]
    pub struct Card {
        pub name: String,
        pub unique_id: Option<String>,
        pub attack: Option<AttackData>,
        pub movement: Option<MovementData>,
    }
}
pub use inner::*;
