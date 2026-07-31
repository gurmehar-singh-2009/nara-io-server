#![feature(yeet_expr)]

use bitcode::{Decode, Encode};
use bytes::{BufMut, Bytes, BytesMut};
use snafu::ResultExt;

pub type HandShake = [u8; 96];

mod errors;
use crate::errors::{DecodeSnafu, InvalidByteSizeSnafu, SharedError};

const PACKET_SEED: u8 = parse_u8(env!("PACKET_SEED"));

const fn parse_u8(s: &str) -> u8 {
    let bytes = s.as_bytes();
    let mut val: u8 = 0;
    let mut i = 0;

    while i < bytes.len() {
        val = val * 10 + (bytes[i] - b'0');
        i += 1;
    }

    val
}

const DUMMY_SIZE_A: usize = ((PACKET_SEED % 3) + 1) as usize;
const DUMMY_SIZE_B: usize = ((PACKET_SEED % 5) + 1) as usize;

#[derive(Encode, Decode)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct JunkData<const N: usize>(pub [u8; N]);

impl<const N: usize> JunkData<N> {
    pub fn random(entropy: u64) -> Self {
        let mut data = [0u8; N];
        let mut state = entropy.wrapping_add(PACKET_SEED as u64);
        let mut i = 0;

        while i < N {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            data[i] = (state & 0xFF) as u8;
            i += 1;
        }

        Self(data)
    }
}

impl<const N: usize> Default for JunkData<N> {
    fn default() -> Self {
        Self::random(0)
    }
}

pub trait Packet: Send + Sync + Encode + for<'de> Decode<'de> {
    const ID: u8;
}

pub trait ClientBoundPacket<'de>: Packet {
    fn encode(self) -> Bytes
    where
        for<'b> &'b Self: Encode,
    {
        let raw_payload = bitcode::encode(&self);
        let mut bytes = BytesMut::with_capacity(1 + raw_payload.len());

        bytes.put_u8(Self::ID);
        bytes.extend_from_slice(&raw_payload);

        bytes.freeze()
    }

    fn decode<P>(data: &'de [u8]) -> Result<P, SharedError>
    where
        P: Packet,
    {
        if data.is_empty() {
            do yeet InvalidByteSizeSnafu.build();
        }

        let decoded = bitcode::decode::<P>(data).context(DecodeSnafu)?;

        Ok(decoded)
    }
}

#[derive(Encode, Decode)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub struct HandshakePacket {
    pub _junk_a: JunkData<DUMMY_SIZE_A>,
    pub handshake: HandShake,
    pub _junk_b: JunkData<DUMMY_SIZE_B>,
}

impl HandshakePacket {
    pub fn new(handshake: HandShake, sequence_or_time: u64) -> Self {
        Self {
            _junk_a: JunkData::random(sequence_or_time),
            handshake,
            _junk_b: JunkData::random(sequence_or_time.wrapping_mul(31)),
        }
    }
}

#[derive(Encode, Decode)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum ClientPacket {
    Handshake(HandshakePacket),
}

#[derive(Encode, Decode)]
#[cfg_attr(debug_assertions, derive(Debug))]
pub enum ServerPacket {
    Handshake(HandshakePacket),
}

macro_rules! register_packets {
    ($( ($index:expr, $struct_name:ident) ),* $(,)?) => {
        $(
            impl Packet for $struct_name {
                const ID: u8 = ($index as u8) ^ PACKET_SEED;
            }
        )*
    };
}

register_packets! {
    (0, HandshakePacket),
}
