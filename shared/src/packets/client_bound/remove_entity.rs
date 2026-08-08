use bitcode::{Decode, Encode};

use crate::{junk_packet, packets::client_bound::EntityType};

junk_packet! {
    pub struct RemoveEntity {
        pub id: u32,
        pub entity_type: EntityType,
    }
}
