//! Modbus request messages. You can use [`zerocopy::IntoBytes`] to convert
//! them into byte buffers for sending.
//!
//! ```
//! # use async_modbus::request::WriteHolding;
//! # use hex_literal::hex;
//! use async_modbus::zerocopy::IntoBytes;
//!
//! let message = WriteHolding::new(0x01, 0x10BC, 12345);
//! assert_eq!(message.as_bytes(), hex!("01 06 10 BC 30 39 98 FC"));
//! ```

use zerocopy::big_endian;

// Re-export generated types with proper names
pub use crate::generated::request_WriteHolding as WriteHolding;
pub use crate::generated::request_ReadHoldings as ReadHoldings;
pub use crate::generated::request_WriteHoldings as WriteHoldings;
pub use crate::generated::request_ReadInputs as ReadInputs;

impl WriteHolding {
    /// Create a new write holding register request
    pub fn new(addr: u8, register: u16, value: u16) -> Self {
        Self::new_inner(addr, register.into(), value.into())
    }
}

impl ReadHoldings {
    /// Create a new read holding registers request
    pub fn new(addr: u8, starting_register: u16, n_registers: u16) -> Self {
        Self::new_inner(addr, starting_register.into(), n_registers.into())
    }
}

impl<const N: usize> WriteHoldings<N> {
    /// Create a new write multiple holding registers request
    pub fn new(addr: u8, starting_register: u16, data: [u16; N]) -> Self {
        Self::new_inner(
            addr,
            starting_register.into(),
            big_endian::U16::new(N as u16),
            (N as u8) * 2,
            data.map(big_endian::U16::new),
        )
    }
}

impl ReadInputs {
    /// Create a new read input registers request
    pub fn new(addr: u8, starting_register: u16, n_registers: u16) -> Self {
        Self::new_inner(addr, starting_register.into(), n_registers.into())
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use zerocopy::IntoBytes;

    use super::*;

    #[test]
    fn test_write_holding_register() {
        let msg = WriteHolding::new(0x01, 0x1001, 0x03E8);
        assert_eq!(msg.as_bytes(), hex!("01 06 10 01 03 E8 DC 74"),);
    }

    #[test]
    fn test_read_holding_registers() {
        let msg = ReadHoldings::new(0x01, 0x1001, 1000);
        assert_eq!(msg.as_bytes(), hex!("01 03 10 01 03 E8 10 74"),);
    }
}
