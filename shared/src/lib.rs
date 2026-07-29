#![feature(yeet_expr)]

use std::error;

use bitcode::{__private::Encoder, Decode, Encode};
use bytes::{BufMut, Bytes, BytesMut};
use snafu::ResultExt;

pub type HandShake = [u8; 96];

use crate::errors::{
    DecodeSnafu, InvalidByteSizeSnafu,
    SharedError::{self},
};

mod errors;
// use errors::{shared_err}

pub trait Packet: Send + Sync + Encode + for<'de> Decode<'de> {
    const ID: u8;
}

pub trait ClientBoundPacket<'de>: Packet {
    fn encode(self) -> Bytes
    where
        for<'b> &'b Self: Encode,
    {
        let encoded = bitcode::encode(&self);
        let mut bytes = BytesMut::with_capacity(encoded.len() + 1);
        bytes.put_u8(Self::ID);
        bytes.extend_from_slice(&encoded);

        bytes.freeze()
    }

    fn decode<P>(&self, data: &'de [u8]) -> Result<P, SharedError>
    where
        P: Packet,
    {
        if data.is_empty() {
            do yeet InvalidByteSizeSnafu.build();
        }

        let d = bitcode::decode::<P>(data).context(DecodeSnafu)?;

        Ok(d)
    }
}

#[derive(Debug, Encode, Decode)]
pub struct HandshakePacket(pub HandShake);

impl Packet for HandshakePacket {
    const ID: u8 = 0;
}

#[derive(Debug, Encode, Decode)]
/// Packets that are targetted to the client.
pub enum ClientPacket {
    Handshake(HandshakePacket),
}

#[derive(Debug, Encode, Decode)]
/// Packets that are targetted to the server.
pub enum ServerPacket {
    Handshake(HandshakePacket),
}

// pub trait ServerBoundPacket {
//     fn encode(&self) -> Bytes {}
// }
