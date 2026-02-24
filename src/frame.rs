use zerocopy::{FromZeros, Immutable, IntoBytes, Unaligned, little_endian, transmute};
use zerocopy_derive::*;

const MAX_FRAME_SIZE: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoBytes, Unaligned, Immutable, FromBytes)]
#[repr(C)]
pub struct Frame<T> {
    server_address: u8,
    pdu: T,
    crc: little_endian::U16,
}

impl<T> Frame<T> {
    pub const fn new(server_address: u8, pdu: T) -> Self {
        Frame {
            server_address,
            pdu,
            crc: little_endian::U16::ZERO,
        }
    }

    fn calculate_crc(&self) -> u16
    where
        T: IntoBytes + Unaligned + Immutable,
    {
        let bytes = self.as_bytes();
        // The last two bytes are the CRC itself
        crate::crc(&bytes[..bytes.len() - 2])
    }

    pub fn update_crc(&mut self)
    where
        T: IntoBytes + Unaligned + Immutable,
    {
        self.crc = self.calculate_crc().into();
    }
}
