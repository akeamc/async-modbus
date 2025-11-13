#![no_std]

#[cfg(feature = "embedded-io")]
pub mod embedded_io;

mod util;

pub mod request;
pub mod response;

pub use util::crc;

#[derive(Debug, thiserror_no_std::Error)]
pub enum Error<Io> {
    /// IO error.
    #[error(transparent)]
    Io(Io),
    /// Unexpected end of file when reading.
    #[error("unexpected end of file")]
    UnexpectedEof,
    /// Invalid CRC checksum.
    #[error("invalid CRC checksum")]
    InvalidCrc,
    /// Unexpected response from the Modbus device.
    #[error("unexpected response from device")]
    UnexpectedResponse,
}

pub struct CrcError;

impl<E> From<CrcError> for Error<E> {
    fn from(_: CrcError) -> Self {
        Error::UnexpectedResponse
    }
}
