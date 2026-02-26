//! Client functions for [`embedded_io_async`]-based IO.

use embedded_io_async::{Read, ReadExactError, Write};
use zerocopy::{FromBytes, Immutable, IntoBytes};

mod generated {
    include!(concat!(env!("OUT_DIR"), "/client.rs"));
}

pub use generated::*;

use crate::pdu::{CrcError, ValidationError};

async fn write_frame<T, E>(mut dst: impl Write<Error = E>, frame: &T) -> Result<(), E>
where
    T: IntoBytes + Immutable,
{
    dst.write_all(frame.as_bytes()).await?;
    dst.flush().await
}

async fn read_frame<T, E>(mut src: impl Read<Error = E>) -> Result<T, ReadExactError<E>>
where
    T: FromBytes + IntoBytes,
{
    let mut message = T::new_zeroed();
    src.read_exact(message.as_mut_bytes()).await?;
    Ok(message)
}

/// Errors that can occur when talking to a Modbus server.
#[derive(Debug, thiserror_no_std::Error)]
pub enum Error<Io> {
    /// IO error.
    #[error(transparent)]
    Io(Io),
    /// Unexpected end of file when reading.
    #[error("unexpected end of file")]
    UnexpectedEof,
    /// Invalid CRC checksum.
    #[error(transparent)]
    Crc(#[from] CrcError),
    /// Unexpected response from the Modbus server.
    #[error("unexpected response from server")]
    UnexpectedResponse,
}

impl<E> From<ValidationError> for Error<E> {
    fn from(e: ValidationError) -> Self {
        match e {
            ValidationError::Crc(crc) => Error::Crc(crc),
            ValidationError::UnexpectedResponse => Error::UnexpectedResponse,
        }
    }
}

impl<E> From<ReadExactError<E>> for Error<E> {
    fn from(e: ReadExactError<E>) -> Self {
        match e {
            ReadExactError::Other(e) => Self::Io(e),
            ReadExactError::UnexpectedEof => Self::UnexpectedEof,
        }
    }
}
