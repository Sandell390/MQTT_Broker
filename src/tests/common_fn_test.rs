#[cfg(test)]
mod tests {
    use crate::common_fn::{bit_operations::{decode_remaining_length, split_byte}, msb_lsb_creater::{create_packet, split_into_msb_lsb}, msb_lsb_reader::get_values};

    #[test]   
    fn test_decode_remaining_length() {
        // Test with a single byte remaining length
        let bytes = [0x00, 0x40]; // 64 in decimal
        assert_eq!(decode_remaining_length(&bytes), Ok(64));

        // Test with a two bytes remaining length
        let bytes = [0x00, 0xC1, 0x02]; // 321 in decimal
        assert_eq!(decode_remaining_length(&bytes), Ok(321));

        // Test with a maximum four bytes remaining length
        let bytes = [0x00, 0xFF, 0xFF, 0xFF, 0x7F]; // 268435455 in decimal
        assert_eq!(decode_remaining_length(&bytes), Ok(268435455));

        // Test with a remaining length that falls in the range of two bytes
        let bytes = [0x00, 0xFF, 0x7F]; // 16383 in decimal
        assert_eq!(decode_remaining_length(&bytes), Ok(16383));

        // Test with a remaining length that falls in the range of three bytes
        let bytes = [0x00, 0x80, 0x80, 0x01]; // 16384 in decimal
        assert_eq!(decode_remaining_length(&bytes), Ok(16384));

        // Test with a remaining length that falls in the range of four bytes
        let bytes = [0x00, 0x80, 0x80, 0x80, 0x01]; // 2097152 in decimal
        assert_eq!(decode_remaining_length(&bytes), Ok(2097152));

        // Test with an empty packet
        let bytes = [0x00];
        assert_eq!(decode_remaining_length(&bytes), Err("Unexpected end of packet"));

        // Test with a malformed remaining length
        let bytes = [0x00, 0x80, 0x80, 0x80, 0x80, 0x01];
        assert_eq!(decode_remaining_length(&bytes), Err("Malformed Remaining Length"));
    }
    #[test]
    fn test_split_byte() {
        // Test with a valid split index
        let byte = 0b10101010;
        assert_eq!(split_byte(&byte, 4), Ok([0b1010, 0b1010]));

        // Test with an invalid split index
        let byte = 0b10101010;
        assert_eq!(split_byte(&byte, 8), Err("split_index is not allowed to more than 7"));

        // Test with a split index of 0
        let byte = 0b10101010;
        assert_eq!(split_byte(&byte, 0), Err("split_index is not allowed to be 0"));
    }

    #[test]
    fn test_create_packet() {
        // Test with a valid string
        let string_value = "Hello, world!";
        let expected = vec![0, 13, 'H' as u8, 'e' as u8, 'l' as u8, 'l' as u8, 'o' as u8, ',' as u8, ' ' as u8, 'w' as u8, 'o' as u8, 'r' as u8, 'l' as u8, 'd' as u8, '!' as u8];
        assert_eq!(create_packet(string_value), Ok(expected));

        // Test with an empty string
        let string_value = "";
        let expected = vec![0, 0];
        assert_eq!(create_packet(string_value), Ok(expected));

        // Test with a string that exceeds the maximum length
        let string_value = "a".repeat(0x10000);
        assert_eq!(create_packet(&string_value), Err("String is too long to create a packet"));
    }

    #[test]
    fn test_split_into_msb_lsb() {
        // Test with a value that fits in one byte
        let value = 0x7F; // 127 in decimal
        assert_eq!(split_into_msb_lsb(value), [0x00, 0x7F]);

        // Test with a value that requires two bytes
        let value = 0x1FF; // 511 in decimal
        assert_eq!(split_into_msb_lsb(value), [0x01, 0xFF]);

        // Test with a value that is exactly two bytes
        let value = 0xFFFF; // 65535 in decimal
        assert_eq!(split_into_msb_lsb(value), [0xFF, 0xFF]);

        // Test with a value of zero
        let value = 0x00; // 0 in decimal
        assert_eq!(split_into_msb_lsb(value), [0x00, 0x00]);
    }

    #[test]
    fn test_get_values() {
        // Test with a valid buffer and reading the string value
        let buffer = &[0x00, 0x05, 0x48, 0x65, 0x6c, 0x6c, 0x6f]; // "Hello" in UTF-8
        let (decimal_value, string_value, stop_index) = get_values(buffer, 0, true).unwrap();
        assert_eq!(decimal_value, 5);
        assert_eq!(string_value, "Hello");
        assert_eq!(stop_index, 7);

        // Test with a valid buffer and not reading the string value
        let buffer = &[0x00, 0x05, 0x48, 0x65, 0x6c, 0x6c, 0x6f]; // "Hello" in UTF-8
        let (decimal_value, string_value, stop_index) = get_values(buffer, 0, false).unwrap();
        assert_eq!(decimal_value, 5);
        assert_eq!(string_value, "");
        assert_eq!(stop_index, 2);

        // Test with a buffer that is too small to read the MSB and LSB
        let buffer = &[0x00];
        assert_eq!(get_values(buffer, 0, true), Err("Buffer is too small to read MSB and LSB"));

        // Test with a buffer that is too small to read the string value
        let buffer = &[0x00, 0x05, 0x48];
        assert_eq!(get_values(buffer, 0, true), Err("Buffer is too small to read the string value"));
    }
}