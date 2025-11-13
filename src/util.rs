/// Calculate CRC for Modbus RTU
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
macro_rules! modbus_message_methods {
    ($function_code:expr, $($field_name:ident: $field_type:ty),*) => {
        #[allow(dead_code)]
        fn new_inner(addr: u8, $($field_name: $field_type),*) -> Self {
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

        /// Calculate the CRC for this message
        pub fn calculate_crc(&self) -> u16 {
            let bytes = self.as_bytes();
            crate::crc(&bytes[..bytes.len() - 2])
        }

        /// Check if the CRC is valid
        pub fn validate_crc(&self) -> Result<(), $crate::CrcError> {
            if self.crc.get() == self.calculate_crc() {
                Ok(())
            } else {
                Err($crate::CrcError)
            }
        }

        /// Get the device address
        pub fn address(&self) -> u8 {
            self.addr
        }

        /// Get the function code
        pub fn function(&self) -> u8 {
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
                pub $field_name: $field_type,
            )*
            crc: zerocopy::little_endian::U16,
        }

        impl $name {
            $crate::util::modbus_message_methods!($function_code, $($field_name: $field_type),*);
        }
    };

    // Variant for messages with const generics
    (
        $(#[$outer:meta])*
        $name:ident<const $generic:ident: $generic_type:ty> {
            function_code: $function_code:expr,
            $(
                $field_name:ident: $field_type:ty
            ),* $(,)?
        }
    ) => {
        $(#[$outer])*
        #[derive(IntoBytes, Immutable, FromBytes, KnownLayout)]
        #[repr(C)]
        pub struct $name<const $generic: $generic_type> {
            addr: u8,
            function: u8,
            $(
                pub $field_name: $field_type,
            )*
            crc: zerocopy::little_endian::U16,
        }

        impl<const $generic: $generic_type> $name<$generic> {
            $crate::util::modbus_message_methods!($function_code, $($field_name: $field_type),*);
        }
    };
}

pub(crate) use {modbus_message, modbus_message_methods};

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    #[test]
    fn crc() {
        assert_eq!(super::crc(&hex!("00 06 00 00 00 17")), 0x15c8);
    }
}
