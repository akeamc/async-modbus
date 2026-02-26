use zerocopy::{Immutable, IntoBytes, Unaligned};

/// Modbus request messages. You can use [`zerocopy::IntoBytes`] to convert
/// them into byte buffers for sending.
///
/// ```
/// # use async_modbus::{Frame, pdu::request::WriteHolding};
/// # use hex_literal::hex;
/// use async_modbus::zerocopy::IntoBytes;
///
/// let pdu = WriteHolding::new().with_register(0x10BCu16.into()).with_value(12345u16.into());
/// let frame = Frame::new(0x01, pdu);
/// assert_eq!(frame.as_bytes(), hex!("01 06 10 BC 30 39 98 FC"));
/// ```
#[allow(unused)]
pub mod request {
    include!(concat!(env!("OUT_DIR"), "/pdu_req.rs"));

    #[cfg(test)]
    mod tests {
        use hex_literal::hex;
        use zerocopy::IntoBytes;

        use crate::frame::FrameBuilder;

        use super::*;

        // #[test]
        // fn test_write_holding_register() {
        //     let msg = WriteHolding::new(0x01, 0x1001, 0x03E8);
        //     assert_eq!(msg.as_bytes(), hex!("01 06 10 01 03 E8 DC 74"),);
        // }

        #[test]
        fn test_read_holding_registers() {
            let frame = FrameBuilder::with_pdu(
                0x01,
                ReadHoldings::new()
                    .with_n_registers(0x03E8.into())
                    .with_starting_register(0x1001.into()),
            )
            .build();

            assert_eq!(frame.as_bytes(), hex!("01 03 10 01 03 E8 10 74"),);
        }
    }
}

pub mod response {
    include!(concat!(env!("OUT_DIR"), "/pdu_res.rs"));
}

pub trait Pdu: Unaligned + Immutable + IntoBytes {
    const FUNCTION_CODE: u8;

    const DEFAULT: Self;
}

pub trait Response<Request>: Pdu {
    type Data;

    fn matches_request(&self, request: &Request) -> bool;

    fn into_data(self) -> Self::Data;
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
