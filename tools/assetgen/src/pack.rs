//! Coverage packing.

/// Packs 8-bit coverage into 4-bit nibbles, high nibble first.
pub fn pack_4bit(values: &[u8]) -> Vec<u8> {
    let mut out = vec![0u8; values.len().div_ceil(2)];
    for (index, &value) in values.iter().enumerate() {
        let nibble = value >> 4;
        if index % 2 == 0 {
            out[index / 2] = nibble << 4;
        } else {
            out[index / 2] |= nibble;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_high_nibble_first() {
        assert_eq!(pack_4bit(&[0x10, 0x20]), vec![0x12]);
        assert_eq!(pack_4bit(&[0xF0]), vec![0xF0]);
        assert_eq!(pack_4bit(&[0x00, 0xFF, 0x80]), vec![0x0F, 0x80]);
    }
}
