use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum SharedError {
    #[snafu(display("Failed to decode data: {source}"))]
    DecodeError {
        source: bitcode::Error,
    },

    InvalidByteSize,
}
