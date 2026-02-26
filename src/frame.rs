use zerocopy::{Immutable, IntoBytes, Unaligned, little_endian};
use zerocopy_derive::*;

use crate::{
    Pdu,
    pdu::{CrcError, Response, ValidationError},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoBytes, Unaligned, Immutable, FromBytes)]
#[repr(C)]
pub struct Frame<T> {
    unit_id: u8,
    pdu: T,
    crc: little_endian::U16,
}

impl<T> Frame<T> {
    pub fn new(unit_id: u8, pdu: T) -> Self
    where
        T: Pdu,
    {
        let mut frame = Self::without_crc(unit_id, pdu);
        frame.update_crc();
        frame
    }

    const fn without_crc(unit_id: u8, pdu: T) -> Self {
        Frame {
            unit_id,
            pdu,
            crc: little_endian::U16::ZERO,
        }
    }

    pub const fn builder(unit_id: u8) -> FrameBuilder<T>
    where
        T: Pdu,
    {
        FrameBuilder::new(unit_id)
    }

    fn calculate_crc(&self) -> u16
    where
        T: IntoBytes + Unaligned + Immutable,
    {
        let bytes = self.as_bytes();
        // The last two bytes are the CRC itself
        crate::crc(&bytes[..bytes.len() - 2])
    }

    fn update_crc(&mut self)
    where
        T: IntoBytes + Unaligned + Immutable,
    {
        self.crc = self.calculate_crc().into();
    }

    pub fn into_data<Request>(self, request: &Frame<Request>) -> Result<T::Data, ValidationError>
    where
        T: Response<Request>,
    {
        if self.calculate_crc() != self.crc.get() {
            return Err(ValidationError::Crc(CrcError));
        }

        if self.unit_id != request.unit_id {
            return Err(ValidationError::UnexpectedResponse);
        }

        if !self.pdu.matches_request(&request.pdu) {
            return Err(ValidationError::UnexpectedResponse);
        }

        Ok(self.pdu.into_data())
    }
}

pub struct FrameBuilder<T> {
    inner: Frame<T>,
}

impl<T: Pdu> FrameBuilder<T> {
    pub const fn with_pdu(unit_id: u8, pdu: T) -> Self {
        Self {
            // crc is calculated later
            inner: Frame::without_crc(unit_id, pdu),
        }
    }

    pub const fn new(unit_id: u8) -> Self {
        Self::with_pdu(unit_id, T::DEFAULT)
    }

    pub fn build_ref(&mut self) -> &mut Frame<T> {
        self.inner.update_crc();
        &mut self.inner
    }

    pub fn build(mut self) -> Frame<T> {
        self.inner.update_crc();
        self.inner
    }

    pub fn pdu_mut(&mut self) -> &mut T {
        &mut self.inner.pdu
    }
}
