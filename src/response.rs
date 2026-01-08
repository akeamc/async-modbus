//! Modbus response messages and validation against requests.
//!
//! Since the responses all implement [`zerocopy::FromBytes`], they can be read
//! directly from a byte buffer. However, this also means that there is no
//! validation of the response data, not even the CRC checksum.

use crate::{ValidationError, request};

use super::util::modbus_message;
use zerocopy::{IntoBytes, big_endian, little_endian};
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

impl<const N: usize> ReadHoldings<N> {
    /// Create a new ReadHoldings response.
    ///
    /// # Panics
    ///
    /// Panics if the number of registers `N` is greater than 127.
    #[inline]
    pub fn new(addr: u8, data: [big_endian::U16; N]) -> Self {
        Self::new_inner(addr, 2 * N as u8, data)
    }

    /// Create a new ReadHoldings response in place.
    #[inline]
    pub fn new_with(addr: u8, f: impl FnOnce(&mut [big_endian::U16; N])) -> Self {
        Self::new_with_inner(addr, |m| f(&mut m.data))
    }

    pub fn data_mut(&mut self) -> &mut [big_endian::U16; N] {
        &mut self.data
    }
}

pub struct ReadHoldingsBuilder<const N: usize>(ReadHoldings<N>);

impl<const N: usize> Default for ReadHoldingsBuilder<N> {
    fn default() -> Self {
        Self::new(0)
    }
}

impl<const N: usize> ReadHoldingsBuilder<N> {
    pub const fn new(addr: u8) -> Self {
        Self(ReadHoldings {
            addr,
            function: ReadHoldings::<N>::FUNCTION,
            data_bytes: 2 * N as u8,
            data: [big_endian::U16::ZERO; N],
            crc: little_endian::U16::ZERO,
        })
    }

    pub fn data_mut(&mut self) -> &mut [big_endian::U16; N] {
        &mut self.0.data
    }

    pub fn finish_ref(&mut self) -> &mut ReadHoldings<N> {
        self.0.update_crc();
        &mut self.0
    }

    pub fn finish(mut self) -> ReadHoldings<N> {
        self.0.update_crc();
        self.0
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

    /// Validate the response against a given request and extract the data on
    /// success.
    fn into_data(self, request: &Request) -> Result<Self::Data, ValidationError>;
}
