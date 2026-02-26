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

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    #[test]
    fn crc() {
        assert_eq!(super::crc(&hex!("00 06 00 00 00 17")), 0x15c8);
    }
}
