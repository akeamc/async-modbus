use zerocopy::{FromBytes, Immutable, IntoBytes, Unaligned, little_endian};
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

    /// Upcast the frame to an [`FrameView`].
    pub fn view(&self) -> FrameView<'_>
    where
        T: IntoBytes + Unaligned + Immutable,
    {
        FrameView {
            buf: self.as_bytes(),
        }
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

/// A frame with an unknown PDU type.
///
/// ```
/// # use hex_literal::hex;
/// # use async_modbus::FrameView;
/// let frame = FrameView::try_from_bytes(&hex!("01 06 00 04 00 02 49 CA")).unwrap();
///
/// assert_eq!(frame.unit_id(), 1);
/// assert!(frame.validate_crc().is_ok());
/// ```
pub struct FrameView<'a> {
    buf: &'a [u8],
}

impl<'a> FrameView<'a> {
    /// Parse a frame from a byte slice. This method does not validate the CRC
    /// or the PDU contents, only that the frame has a valid length.
    pub fn try_from_bytes(buf: &'a [u8]) -> Option<Self> {
        if (4..=256).contains(&buf.len()) {
            Some(Self { buf })
        } else {
            None
        }
    }

    /// The unit ID of the frame.
    pub const fn unit_id(&self) -> u8 {
        self.buf[0]
    }

    /// The CRC sent with the frame. This struct does not guarantee that the
    /// CRC is valid; be sure to check it with [`FrameView::validate_crc`].
    pub const fn crc(&self) -> u16 {
        u16::from_le_bytes([self.buf[self.buf.len() - 2], self.buf[self.buf.len() - 1]])
    }

    fn calculate_crc(&self) -> u16 {
        crate::crc(&self.buf[..self.buf.len() - 2])
    }

    /// Returns the wrapped PDU if the CRC is valid.
    pub fn pdu(self) -> Result<&'a PduView, CrcError> {
        self.validate_crc()?;
        Ok(
            PduView::ref_from_bytes(&self.buf[1..self.buf.len() - 2])
                .expect("PduView is Unaligned"),
        )
    }

    /// Validate the CRC of the frame.
    ///
    /// Note that [`FrameView::pdu`] validates the CRC before returning
    /// the PDU, rendering this method unnecessary to call directly in most
    /// cases.
    pub fn validate_crc(&self) -> Result<(), CrcError> {
        if self.calculate_crc() == self.crc() {
            Ok(())
        } else {
            Err(CrcError)
        }
    }
}

/// The most generic Modbus PDU, containing a function code and data payload.
#[derive(Debug, FromBytes, KnownLayout, Immutable, IntoBytes, Unaligned)]
#[repr(C)]
pub struct PduView {
    /// The function code of the PDU.
    pub function_code: u8,
    /// The data payload of the PDU.
    pub data: [u8],
}

impl PduView {
    /// Parse into a concrete PDU type. Will return `None` upon a function code
    /// or size mismatch.
    ///
    /// ```
    /// # use hex_literal::hex;
    /// # use async_modbus::{FrameView, PduView};
    /// # use async_modbus::pdu::response;
    /// let pdu = FrameView::try_from_bytes(&hex!("01 03 04 00 01 76 3B CC 40"))
    ///     .unwrap()
    ///     .pdu()
    ///     .unwrap();
    ///
    /// let read_holdings = pdu.parse::<response::ReadHoldings::<2>>().unwrap();
    ///
    /// assert_eq!(read_holdings.byte_count(), 2);
    /// assert_eq!(read_holdings.data().map(|d| d.get()), [0x00_01, 0x76_3B]);
    /// ```
    #[inline]
    pub fn parse<T: Pdu>(&self) -> Option<&T> {
        if self.function_code == T::FUNCTION_CODE {
            println!("{:x?}", self.as_bytes());
            T::try_ref_from_bytes(self.as_bytes()).ok()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;

    #[test]
    fn test_length_validation() {
        assert!(FrameView::try_from_bytes(&hex!()).is_none());
        assert!(FrameView::try_from_bytes(&hex!("00 00 00")).is_none());
        assert!(FrameView::try_from_bytes(&hex!("00 00 00 00")).is_some());
        assert!(FrameView::try_from_bytes(&[0; 256]).is_some());
        assert!(FrameView::try_from_bytes(&[0; 257]).is_none());
    }
}
