//! PDUs in Modbus are everything in between the unit ID and CRC.

use zerocopy::{Immutable, IntoBytes, KnownLayout, TryFromBytes, Unaligned};

/// Modbus request PDUs. Wrap them in a [`Frame`](crate::Frame) to give them a
/// unit ID and CRC.
///
/// ```
/// # use async_modbus::{Frame, pdu::request::WriteHolding};
/// # use hex_literal::hex;
/// use async_modbus::zerocopy::IntoBytes;
///
/// let pdu = WriteHolding::new().with_register(0x10BC).with_value(12345);
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
                    .with_n_registers(0x03E8)
                    .with_starting_register(0x1001),
            )
            .build();

            assert_eq!(frame.as_bytes(), hex!("01 03 10 01 03 E8 10 74"),);
        }
    }
}

/// Response PDUs!
pub mod response {
    include!(concat!(env!("OUT_DIR"), "/pdu_res.rs"));
}

/// A Protocol Data Unit (PDU) contains the function code and data.
pub trait Pdu: Unaligned + Immutable + IntoBytes + TryFromBytes + KnownLayout {
    /// The function code of the PDU.
    const FUNCTION_CODE: u8;

    /// Default value for the PDU. Might be all zeros, but not always:
    ///
    /// For instance, [`response::ReadHoldings`] is always initialized with a
    /// default byte count of `2 * N`.
    const DEFAULT: Self;
}

/// The [`Response`] trait is used to validate a response against a request,
/// essentially to ensure that the response corresponds to the request.
///
/// It also provides a method to convert the response into its data payload.
pub trait Response<Request>: Pdu {
    /// The data payload of the response.
    type Data;

    /// Validates the response against the request.
    fn matches_request(&self, request: &Request) -> bool;

    /// Converts the response into its data payload.
    fn into_data(self) -> Self::Data;
}

/// Error indicating a CRC validation failure.
#[derive(Debug, Clone, Copy, thiserror_no_std::Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[error("CRC validation failed")]
pub struct CrcError;

/// Errors that can occur when validating a Modbus response.
#[derive(Debug, thiserror_no_std::Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ValidationError {
    /// CRC validation failed.
    #[error(transparent)]
    Crc(#[from] CrcError),
    /// The response did not match the request.
    #[error("unexpected response")]
    UnexpectedResponse,
}
