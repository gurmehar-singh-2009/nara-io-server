// we send this initially which contains ALL the tanks
//

use std::collections::HashMap;

use bitcode::{Decode, Encode};

use crate::{
    junk_packet,
    packets::client_bound::{EntityType, TankSpec},
};

junk_packet! {
    pub struct TankCatalog {
        data: HashMap<String, TankSpec>,
    }
}
