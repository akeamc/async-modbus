//! Modbus response messages and validation against requests.
//!
//! Since the responses all implement [`zerocopy::FromBytes`], they can be read
//! directly from a byte buffer. However, this also means that there is no
//! validation of the response data, not even the CRC checksum.

use crate::{ValidationError, request};

use super::util::modbus_message;
use zerocopy::{IntoBytes, big_endian};
use zerocopy_derive::*;

modbus_message! {
    /// Write single holding register response
    WriteHolding {
        function_code: 0x06,
        register: big_endian::U16,
        value: big_endian::U16,
    }
}

impl Response<request::WriteHolding> for WriteHolding {
    type Data = ();

    fn into_data(self, req: &request::WriteHolding) -> Result<(), ValidationError> {
        self.validate_crc()?;

        if self.address() == req.address()
            && self.function() == req.function()
            && self.register == req.register
            && self.value == req.value
        {
            Ok(())
        } else {
            Err(ValidationError::UnexpectedResponse)
        }
    }
}

modbus_message! {
    /// Read holding registers response
    ReadHoldings<const N: usize> {
        function_code: 0x03,
        data_bytes: u8,
        data: [big_endian::U16; N],
    }
}

impl<const N: usize> Response<request::ReadHoldings> for ReadHoldings<N> {
    type Data = [big_endian::U16; N];

    fn into_data(self, req: &request::ReadHoldings) -> Result<Self::Data, ValidationError> {
        self.validate_crc()?;

        if self.address() == req.address()
            && self.function() == req.function()
            && self.data_bytes == 2 * req.n_registers.get() as u8
        {
            Ok(self.data)
        } else {
            Err(ValidationError::UnexpectedResponse)
        }
    }
}

modbus_message! {
    /// Write multiple holding registers response
    WriteHoldings {
        function_code: 0x10,
        starting_register: big_endian::U16,
        n_registers: big_endian::U16,
    }
}

impl<const N: usize> Response<request::WriteHoldings<N>> for WriteHoldings {
    type Data = ();

    fn into_data(self, req: &request::WriteHoldings<N>) -> Result<(), ValidationError> {
        self.validate_crc()?;

        if self.address() == req.address()
            && self.function() == req.function()
            && self.starting_register == req.starting_register
            && self.n_registers == req.n_registers
        {
            Ok(())
        } else {
            Err(ValidationError::UnexpectedResponse)
        }
    }
}

modbus_message! {
    /// Read input registers response
    ReadInputs<const N: usize> {
        function_code: 0x04,
        data_bytes: u8,
        data: [big_endian::U16; N],
    }
}

impl<const N: usize> Response<request::ReadInputs> for ReadInputs<N> {
    type Data = [big_endian::U16; N];

    fn into_data(self, req: &request::ReadInputs) -> Result<Self::Data, ValidationError> {
        self.validate_crc()?;

        if self.address() == req.address()
            && self.function() == req.function()
            && self.data_bytes == 2 * req.n_registers.get() as u8
        {
            Ok(self.data)
        } else {
            Err(ValidationError::UnexpectedResponse)
        }
    }
}

/// Trait for Modbus response messages that can be validated against requests.
pub trait Response<Request> {
    /// The type of data extracted from the response.
    type Data;

    /// Validate the response against the given request.
    fn into_data(self, request: &Request) -> Result<Self::Data, ValidationError>;
}
