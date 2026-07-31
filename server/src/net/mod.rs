use ed25519_dalek::Signer;
use std::{marker::ConstParamTy_, time::UNIX_EPOCH};

use chacha20poly1305::{Key, KeyInit, Nonce, aead::Aead};
use getrandom::{SysRng, rand_core::UnwrapErr};
use sha2::Sha256;
use shared::{
    ClientPacket::{self},
    HandShake, HandshakePacket,
};
use snafu::ResultExt;
use tokio::sync::mpsc::UnboundedSender;
use x25519_dalek::{EphemeralSecret, PublicKey};

use crate::errors::{ChannelEntrySnafu, CipherEncryptSnafu, ServerError};

#[derive(PartialEq, Eq)]
pub enum ConnectionState {
    Handshaking,
    Authenticated,
}

impl ConstParamTy_ for ConnectionState {}

#[derive(Debug)]
pub struct ClientConnection<const S: ConnectionState> {
    send_tx: UnboundedSender<Vec<u8>>,

    /// Send/recv nonce counter for chacha20poly1305.
    send_nonce_count: u32,
    recv_nonce_count: u32,

    /// The chacha20poly1305 ciphers.
    send_cipher: Option<chacha20poly1305::ChaCha20Poly1305>,
    recv_ciper: Option<chacha20poly1305::ChaCha20Poly1305>,

    /// The player ID associated with this connection struct.
    id: u32,
}

// TODO: Setup ChaCha20-Poly1305.
impl ClientConnection<{ ConnectionState::Handshaking }> {
    pub fn new(id: u32, send_tx: UnboundedSender<Vec<u8>>) -> Self {
        Self {
            send_tx,
            send_nonce_count: 0,
            recv_nonce_count: 0,
            send_cipher: None,
            recv_ciper: None,
            id,
        }
    }

    /// Responds to the client's initiated Handshake Packet.
    /// Performs Diffie Hellman on their public key using our secret key.
    /// We then send back our public key so the client can do those same steps. Voila, we come to the same secret!
    pub fn respond_handshake(
        mut self,
        their_public: &PublicKey,
        server_identity_key: &ed25519_dalek::SigningKey,
    ) -> Result<ClientConnection<{ ConnectionState::Authenticated }>, ServerError> {
        let mut rng = UnwrapErr(SysRng);
        // Create public, private key combo.
        let my_secret = EphemeralSecret::random_from_rng(&mut rng);
        let my_public = PublicKey::from(&my_secret);

        let public_key_signature = server_identity_key.sign(my_public.as_bytes());

        let shared_secret = my_secret.diffie_hellman(their_public);

        let hkdf = hkdf::Hkdf::<Sha256>::new(None, shared_secret.as_bytes());

        // We don't need the shared secret anymore.
        drop(shared_secret);

        let mut client_to_server = [0u8; 32];
        let mut server_to_client = [0u8; 32];

        // TODO: CHANGE "key".
        hkdf.expand("key 1".as_bytes(), &mut client_to_server)
            .unwrap();
        hkdf.expand("key 2".as_bytes(), &mut server_to_client)
            .unwrap();

        // Construct chacha20poly1305 ciphers using the session key.
        let send_cipher = chacha20poly1305::ChaCha20Poly1305::new(
            &Key::try_from(client_to_server.as_slice()).unwrap(),
        );
        let recv_cipher = chacha20poly1305::ChaCha20Poly1305::new(
            &Key::try_from(server_to_client.as_slice()).unwrap(),
        );

        self.send_cipher = Some(send_cipher);
        self.recv_ciper = Some(recv_cipher);

        let mut handshake = [0u8; 96];
        handshake[..32].copy_from_slice(&my_public.to_bytes());
        handshake[32..96].copy_from_slice(&public_key_signature.to_bytes());

        let handshake_packet = ClientPacket::Handshake(HandshakePacket::new(
            handshake,
            std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        ));
        // self.send_tx
        //     .send(handshake_packet)
        //     .context(ChannelEntrySnafu { id: self.id })?;
        self.send(handshake_packet)?;

        Ok(ClientConnection {
            send_tx: self.send_tx,
            send_nonce_count: self.send_nonce_count,
            recv_nonce_count: self.recv_nonce_count,
            send_cipher: self.send_cipher,
            recv_ciper: self.recv_ciper,
            id: self.id,
        })
    }

    /// This fn needs to exist in this stage to send the handshake packet.
    /// But should not be accessed otherwise, wish there was a nicer way.
    fn send(&mut self, data: ClientPacket) -> Result<(), ServerError> {
        let serialized = bitcode::encode(&data);

        if let Some(send_cipher) = &mut self.send_cipher {
            let encoded = send_cipher
                .encrypt(
                    &ClientConnection::nonce(self.send_nonce_count),
                    serialized.as_slice(),
                )
                .context(CipherEncryptSnafu)?;

            self.send_tx
                .send(encoded)
                .context(ChannelEntrySnafu { id: self.id })?;

            self.send_nonce_count += 1;
        } else {
            do yeet ServerError::CipherNotInitialized
        }

        Ok(())
    }
}

impl ClientConnection<{ ConnectionState::Authenticated }> {
    /// Produces the nonce.
    fn nonce(nonce_counter: u32) -> Nonce {
        // Nonce is 12 bytes.
        let mut bytes = [0u8; 12];
        bytes[..4].copy_from_slice(&nonce_counter.to_be_bytes());
        Nonce::from(bytes)
    }

    pub fn send(&mut self, data: ClientPacket) -> Result<(), ServerError> {
        let serialized = bitcode::encode(&data);

        if let Some(send_cipher) = &mut self.send_cipher {
            let encoded = send_cipher
                .encrypt(
                    &ClientConnection::nonce(self.send_nonce_count),
                    serialized.as_slice(),
                )
                .context(CipherEncryptSnafu)?;

            self.send_tx
                .send(encoded)
                .context(ChannelEntrySnafu { id: self.id })?;

            self.send_nonce_count += 1;
        } else {
            do yeet ServerError::CipherNotInitialized
        }

        Ok(())
    }
}

// impl<const S: ConnectionState> ClientConnection<S> {
//     pub fn new(id: u32, send_tx: UnboundedSender<Vec<u8>>) -> Self {
//         Self {
//             send_tx,
//             send_nonce_count: 0,
//             recv_nonce_count: 0,
//             send_cipher: None,
//             recv_ciper: None,
//             id,
//         }
//     }
// }
