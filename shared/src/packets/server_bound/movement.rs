use bitcode::{Decode, Encode};

use crate::junk_packet;

junk_packet! {
    pub struct MovementPacket {
        pub dir: Option<f32>,
    }
}
