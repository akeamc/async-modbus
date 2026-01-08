/// CRC for Modbus RTU messages.
pub fn crc(data: &[u8]) -> u16 {
    let mut crc = 0xffff;

    for &byte in data {
        crc ^= u16::from(byte);
        for _ in 0..8 {
            if (crc & 0x0001) != 0 {
                crc >>= 1;
                crc ^= 0xa001;
            } else {
                crc >>= 1;
            }
        }
    }

    crc
}

/// Macro to generate common methods for Modbus message types
macro_rules! modbus_message_impl {
    ($function_code:expr, $($field_name:ident: $field_type:ty),*) => {
        $crate::util::modbus_message_impl!($function_code, $($field_name: $field_type),*; );
    };
    ($function_code:expr, $($field_name:ident: $field_type:ty),*; $($prologue:stmt)*) => {
        #[allow(dead_code)]
        pub(crate) const FUNCTION: u8 = $function_code;

        #[allow(dead_code)]
        #[inline]
        fn new_inner(addr: u8, $($field_name: $field_type),*) -> Self {
            $($prologue)*

            let mut message = Self {
                addr,
                function: $function_code,
                $(
                    $field_name,
                )*
                crc: Default::default(),
            };

            message.crc = message.calculate_crc().into();
            message
        }

        #[allow(dead_code)]
        fn new_with_inner(addr: u8, f: impl FnOnce(&mut Self)) -> Self {
            $($prologue)*

            let mut message = <Self as zerocopy::FromZeros>::new_zeroed();
            message.addr = addr;
            message.function = $function_code;
            f(&mut message);
            message.crc = message.calculate_crc().into();
            message
        }

        pub(crate) fn calculate_crc(&self) -> u16 {
            let bytes = self.as_bytes();
            // The last two bytes are the CRC itself
            crate::crc(&bytes[..bytes.len() - 2])
        }

        /// Check if the CRC is valid.
        pub fn validate_crc(&self) -> Result<(), $crate::CrcError> {
            if self.crc.get() == self.calculate_crc() {
                Ok(())
            } else {
                Err($crate::CrcError)
            }
        }

        /// Update the CRC from the current message.
        pub fn update_crc(&mut self) {
            self.crc = self.calculate_crc().into();
        }

        /// Get the device address.
        pub fn address(&self) -> u8 {
            self.addr
        }

        /// Get the function code.
        pub(crate) fn function(&self) -> u8 {
            self.function
        }
    };
}

/// Macro to generate Modbus message types
macro_rules! modbus_message {
    (
        $(#[$outer:meta])*
        $name:ident {
            function_code: $function_code:expr,
            $(
                $field_name:ident: $field_type:ty
            ),* $(,)?
        }
    ) => {
        $(#[$outer])*
        #[derive(IntoBytes, Immutable, FromBytes, KnownLayout)]
        #[repr(C)]
        pub struct $name {
            addr: u8,
            function: u8,
            $(
                pub(crate) $field_name: $field_type,
            )*
            crc: zerocopy::little_endian::U16,
        }

        impl $name {
            $crate::util::modbus_message_impl!($function_code, $($field_name: $field_type),*);
        }
    };

    // Variant for messages with const generics
    (
        $(#[$outer:meta])*
        $name:ident<const N: usize> {
            function_code: $function_code:expr,
            $(
                $field_name:ident: $field_type:ty
            ),* $(,)?
        }
    ) => {
        $(#[$outer])*
        #[derive(IntoBytes, Immutable, FromBytes, KnownLayout)]
        #[repr(C)]
        pub struct $name<const N: usize> {
            addr: u8,
            function: u8,
            $(
                pub(crate) $field_name: $field_type,
            )*
            crc: zerocopy::little_endian::U16,
        }

        impl<const N: usize> $name<N> {
            $crate::util::modbus_message_impl!(
                $function_code,
                $($field_name: $field_type),*;
                const { assert!(N <= 127, "N must be less than or equal to 127") }
            );
        }
    };
}

pub(crate) use {modbus_message, modbus_message_impl};

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    #[test]
    fn crc() {
        assert_eq!(super::crc(&hex!("00 06 00 00 00 17")), 0x15c8);
    }
}
