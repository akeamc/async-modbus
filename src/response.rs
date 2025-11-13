use crate::{Error, request};

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

impl WriteHolding {
    /// Check if this response matches the given request
    pub fn validate<E>(&self, req: &request::WriteHolding) -> Result<(), Error<E>> {
        self.validate_crc()?;

        if self.address() == req.address()
            && self.function() == req.function()
            && self.register == req.register
            && self.value == req.value
        {
            Ok(())
        } else {
            Err(Error::UnexpectedResponse)
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
    /// Check if this response matches the given request
    pub fn validate<E>(&self, req: &request::ReadHoldings) -> Result<(), Error<E>> {
        self.validate_crc()?;

        if self.address() == req.address()
            && self.function() == req.function()
            && self.data_bytes == 2 * req.n_registers.get() as u8
        {
            Ok(())
        } else {
            Err(Error::UnexpectedResponse)
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

impl WriteHoldings {
    /// Check if this response matches the given request
    pub fn validate<const N: usize, E>(
        &self,
        req: &request::WriteHoldings<N>,
    ) -> Result<(), Error<E>> {
        self.validate_crc()?;

        if self.address() == req.address()
            && self.function() == req.function()
            && self.starting_register == req.starting_register
            && self.n_registers == req.n_registers
        {
            Ok(())
        } else {
            Err(Error::UnexpectedResponse)
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

impl<const N: usize> ReadInputs<N> {
    pub fn validate<E>(&self, req: &request::ReadInputs) -> Result<(), Error<E>> {
        self.validate_crc()?;

        if self.address() == req.address()
            && self.function() == req.function()
            && self.data_bytes == 2 * req.n_registers.get() as u8
        {
            Ok(())
        } else {
            Err(Error::UnexpectedResponse)
        }
    }
}
