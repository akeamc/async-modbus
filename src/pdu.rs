use crate::Frame;
use zerocopy_derive::*;
#[derive(Debug, Clone, FromBytes, IntoBytes, Immutable, Unaligned)]
#[repr(C)]
pub struct ReadCoils<const N: usize> {
    function_code: u8,
}
impl<const N: usize> ReadCoils<N> {
    pub const FUNCTION_CODE: u8 = 1u8;
    pub const fn new() -> Self {
        Self {
            function_code: Self::FUNCTION_CODE,
        }
    }
}
pub struct ReadCoilsBuilder<const N: usize>(Frame<ReadCoils<N>>);
impl<const N: usize> ReadCoilsBuilder<N> {
    pub const fn new(server_address: u8) -> Self {
        Self(Frame::new(server_address, <ReadCoils<N>>::new()))
    }
}
#[derive(Debug, Clone, FromBytes, IntoBytes, Immutable, Unaligned)]
#[repr(C)]
pub struct ReadDiscreteInputs<const N: usize> {
    function_code: u8,
}
impl<const N: usize> ReadDiscreteInputs<N> {
    pub const FUNCTION_CODE: u8 = 2u8;
    pub const fn new() -> Self {
        Self {
            function_code: Self::FUNCTION_CODE,
        }
    }
}
pub struct ReadDiscreteInputsBuilder<const N: usize>(Frame<ReadDiscreteInputs<N>>);
impl<const N: usize> ReadDiscreteInputsBuilder<N> {
    pub const fn new(server_address: u8) -> Self {
        Self(Frame::new(server_address, <ReadDiscreteInputs<N>>::new()))
    }
}
#[derive(Debug, Clone, FromBytes, IntoBytes, Immutable, Unaligned)]
#[repr(C)]
pub struct ReadHoldingRegisters<const N: usize> {
    function_code: u8,
}
impl<const N: usize> ReadHoldingRegisters<N> {
    pub const FUNCTION_CODE: u8 = 3u8;
    pub const fn new() -> Self {
        Self {
            function_code: Self::FUNCTION_CODE,
        }
    }
}
pub struct ReadHoldingRegistersBuilder<const N: usize>(Frame<ReadHoldingRegisters<N>>);
impl<const N: usize> ReadHoldingRegistersBuilder<N> {
    pub const fn new(server_address: u8) -> Self {
        Self(Frame::new(server_address, <ReadHoldingRegisters<N>>::new()))
    }
}
#[derive(Debug, Clone, FromBytes, IntoBytes, Immutable, Unaligned)]
#[repr(C)]
pub struct ReadInputRegisters<const N: usize> {
    function_code: u8,
}
impl<const N: usize> ReadInputRegisters<N> {
    pub const FUNCTION_CODE: u8 = 4u8;
    pub const fn new() -> Self {
        Self {
            function_code: Self::FUNCTION_CODE,
        }
    }
}
pub struct ReadInputRegistersBuilder<const N: usize>(Frame<ReadInputRegisters<N>>);
impl<const N: usize> ReadInputRegistersBuilder<N> {
    pub const fn new(server_address: u8) -> Self {
        Self(Frame::new(server_address, <ReadInputRegisters<N>>::new()))
    }
}
