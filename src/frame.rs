use zerocopy::{Immutable, IntoBytes, Unaligned, little_endian};
use zerocopy_derive::*;

use crate::{
    Pdu,
    pdu::{CrcError, Response, ValidationError},
};

/// A complete Modbus frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoBytes, Unaligned, Immutable, FromBytes)]
#[repr(C)]
pub struct Frame<T> {
    unit_id: u8,
    pdu: T,
    crc: little_endian::U16,
}

impl<T> Frame<T> {
    /// Creates a new frame with the given unit ID and PDU and calculates the
    /// CRC.
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

    /// Creates a new [`FrameBuilder`].
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

    /// Validate the frame against the given request, returning the data if valid.
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

/// A builder for [`Frame`]s.
#[derive(Debug)]
pub struct FrameBuilder<T> {
    inner: Frame<T>,
}

impl<T: Pdu> FrameBuilder<T> {
    /// Creates a new builder with the given unit ID and PDU.
    ///
    /// This is different from [`Frame::new`] in that no CRC is calculated at
    /// this point.
    pub const fn with_pdu(unit_id: u8, pdu: T) -> Self {
        Self {
            // crc is calculated later
            inner: Frame::without_crc(unit_id, pdu),
        }
    }

    /// Creates a new builder with the given unit ID and default PDU value.
    pub const fn new(unit_id: u8) -> Self {
        Self::with_pdu(unit_id, T::DEFAULT)
    }

    /// Changes the unit ID.
    pub const fn set_unit_id(&mut self, unit_id: u8) {
        self.inner.unit_id = unit_id;
    }

    /// Build a frame but don't move it out of the builder, so that the builder
    /// can be recycled.
    pub fn build_ref(&mut self) -> &mut Frame<T> {
        self.inner.update_crc();
        &mut self.inner
    }

    /// Build a frame (calculate its CRC) and move it out of the builder.
    pub fn build(mut self) -> Frame<T> {
        self.inner.update_crc();
        self.inner
    }

    /// Access the inner PDU mutably.
    pub fn pdu_mut(&mut self) -> &mut T {
        &mut self.inner.pdu
    }
}

impl<T: Pdu> Default for FrameBuilder<T> {
    fn default() -> Self {
        Self::new(0)
    }
}
