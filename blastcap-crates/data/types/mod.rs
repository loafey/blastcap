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
        Ray,
    }

    #[derive(
        Debug, Deserialize, Serialize, Clone, PartialEq, Archive, rkyv::Deserialize, rkyv::Serialize,
    )]
    pub struct AttackData {
        pub damage: i32,
        pub projectile_speed: Option<f32>,
        pub range: usize,
        pub r#type: AttackType,
        pub particle_location: Option<f32>,
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
