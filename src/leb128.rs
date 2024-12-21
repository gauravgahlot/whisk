pub(crate) fn decode(bytes: &[u8]) -> (u32, usize) {
    let mut result = 0;
    let mut shift = 0;
    let mut count = 0;

    for &byte in bytes {
        result |= ((byte & 0x7F) as u32) << shift; // Take 7 bits and shift them into position
        shift += 7;
        count += 1;

        if byte & 0x80 == 0 {
            // If MSB is 0, we've reached the last byte
            break;
        }
    }

    (result, count) // Return the decoded value and the number of bytes used
}

#[cfg(test)]
mod tests {
    use super::decode;

    #[test]
    fn test_decode() {
        let data = [0x7F];
        let (value, size) = decode(&data);
        assert_eq!(value, 127);
        assert_eq!(size, 1);

        let data = vec![0xE5, 0x8E, 0x26];
        let (value, size) = decode(&data);
        assert_eq!(value, 624485);
        assert_eq!(size, 3);
    }
}
