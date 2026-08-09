// auto fire toggles on/off
// client handles this
// same packet used for actual auto fire and just holding mouse

use bitcode::{Decode, Encode};

use crate::junk_packet;

junk_packet! {
    pub struct AutoFirePacket {
        pub enabled: bool,
    }
}
