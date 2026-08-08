use bitcode::{Decode, Encode};

// TODO LIST:
//
// - make add_entity and remove_entity work in bulk (ie, 1 packet will send ALL
//   player AND bullet AND shapes) for both add, remove, update

mod add_entity;
mod remove_entity;
mod update_component;

pub use add_entity::AddEntityPacket;

#[derive(bitcode::Encode, bitcode::Decode, Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityType {
    Player = 0,
    Shape = 1,
    Bullet = 2,
}
