use snafu::prelude::*;
use tokio::sync::mpsc::error::SendError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum ServerError {
    #[snafu(display("Failed to push packet to send channel for {id}"))]
    ChannelEntryError { id: u32, source: SendError<Vec<u8>> },

    #[snafu(display("Failed to encrypt packet"))]
    CipherEncrypt { source: chacha20poly1305::Error },

    #[snafu(display("Cipher not initialized"))]
    CipherNotInitialized,

    #[snafu(display("Handshake failure"))]
    HandshakeFailure,
}
