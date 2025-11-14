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

/// Error indicating a CRC validation failure.
#[derive(Debug, Clone, Copy, thiserror_no_std::Error)]
#[error("CRC validation failed")]
pub struct CrcError;

/// Errors that can occur when validating a Modbus response.
#[derive(Debug, thiserror_no_std::Error)]
pub enum ValidationError {
    /// CRC validation failed.
    #[error(transparent)]
    Crc(#[from] CrcError),
    /// The response did not match the request.
    #[error("unexpected response")]
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
