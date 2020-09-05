use std::convert::{From, TryFrom};
use std::error::Error;
use std::fmt;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct ChunkType {
    bytes: [u8; 4],
}

// Many "unused" methods are used in tests
#[allow(dead_code)]
impl ChunkType {
    /// Must be in ASCII A-Z or a-z (decimal 65-90 and 97-122).
    fn is_valid_byte(b: u8) -> bool {
        (65 <= b && b <= 90) || (97 <= b && b <= 122)
    }

    pub fn bytes(&self) -> [u8; 4] {
        self.bytes
    }

    /// Is the nth bit (from the right, *counting from 0*) zero?
    fn bit_is_zero(bit: u8, n: u8) -> bool {
        // Let's say bit = 117 and n = 5.
        // Mask off the 5th bit, which results in either 0 (not set) or 16
        // (set). Then check if it's 0.
        // For example, for "u" = 117, where the 5th bit is 1:
        //   01110101 = 117 in binary
        // & 00100000 = 1 << 5 = 32
        //   --^-----
        //   00100000 = 16
        let mask = 1 << n;
        bit & mask == 0
    }

    /// A chunk is critical if the ancillary bit is 0.
    /// The ancillary bit is the 5th bit of the 0th byte.
    fn is_critical(&self) -> bool {
        Self::bit_is_zero(self.bytes[0], 5)
    }

    /// A chunk is public if the private bit is 0.
    /// The private bit is the 5th bit of the 1st byte.
    fn is_public(&self) -> bool {
        Self::bit_is_zero(self.bytes[1], 5)
    }

    // The reserved bit is the 5th bit of the 2nd byte.
    /// It has no current meaning, but from the spec
    /// http://www.libpng.org/pub/png/spec/1.2/PNG-Structure.html:
    /// > Must be 0 (uppercase) in files conforming to this version of PNG.
    /// > The significance of the case of the third letter of the chunk name is reserved for possible future expansion
    fn is_reserved_bit_valid(&self) -> bool {
        Self::bit_is_zero(self.bytes[2], 5)
    }

    /// A chunk is safe to copy if the safe-to-copy bit is 1.
    /// The safe-to-copy bit is the 5th bit of the 3rd byte.
    fn is_safe_to_copy(&self) -> bool {
        !Self::bit_is_zero(self.bytes[3], 5)
    }

    /// Is the reserved bit valid?
    fn is_valid(&self) -> bool {
        self.is_reserved_bit_valid()
    }
}

impl fmt::Display for ChunkType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for b in &self.bytes {
            write!(f, "{}", char::from(*b))?;
        }
        Ok(())
    }
}
#[derive(Debug)]
pub enum BadChunkTypeError {
    /// We found a bad byte while decoding. The u8 is the first invalid byte found.
    BadByte(u8),
    /// The chunk type to be decoded was the wrong size. The usize is the received size.
    BadLength(usize),
}
impl fmt::Display for BadChunkTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadByte(byte) => write!(f, "Bad byte: {byte} ({byte:b})", byte = byte),
            Self::BadLength(len) => write!(f, "Bad length: {} (expected 4)", len),
        }
    }
}
impl Error for BadChunkTypeError {}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = crate::Error;

    fn try_from(bytes: [u8; 4]) -> Result<Self, Self::Error> {
        for byte in bytes.iter() {
            if !Self::is_valid_byte(*byte) {
                return Err(Box::new(BadChunkTypeError::BadByte(*byte)));
            }
        }
        Ok(ChunkType { bytes })
    }
}

impl FromStr for ChunkType {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 {
            return Err(Box::new(BadChunkTypeError::BadLength(s.len())));
        }

        let mut vec: [u8; 4] = [0; 4];

        for (index, byte) in s.as_bytes().iter().enumerate() {
            if Self::is_valid_byte(*byte) {
                vec[index] = *byte;
            } else {
                return Err(Box::new(BadChunkTypeError::BadByte(*byte)));
            }
        }
        Ok(ChunkType { bytes: vec })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
