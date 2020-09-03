use crate::chunk_type::ChunkType;
use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::fmt;
use std::fmt::Display;

const MAXIMUM_LENGTH: u32 = (1 << 31) - 1;

// Allow dead code since some functions are only called in tests
#[allow(dead_code)]
/// Each chunk consists of four parts: length, chunk type, chunk data, and CRC.
struct Chunk {
    /// A 4-byte unsigned integer giving the number of bytes in the chunk's data
    /// field. The length counts *only* the data field, *not* itself, the chunk
    /// type code, or the CRC. Zero is a valid length. Although encoders and
    /// decoders should treat the length as unsigned, its value must not exceed
    /// 2^31-1 bytes. (Note that 1 << 31 == 2^31.)
    length: u32,

    /// A 4-byte chunk type code. For convenience in description and in examining PNG
    /// files, type codes are restricted to consist of uppercase and lowercase ASCII
    /// letters (A-Z and a-z, or 65-90 and 97-122 decimal). However, encoders and
    /// decoders must treat the codes as fixed binary values, not character strings.
    /// For example, it would not be correct to represent the type code IDAT by the
    /// EBCDIC equivalents of those letters.
    chunk_type: ChunkType,

    /// The data bytes appropriate to the chunk type, if any. This field can be of
    /// zero length.
    chunk_data: Vec<u8>,

    /// A 4-byte CRC (Cyclic Redundancy Check) calculated on the preceding bytes in
    /// the chunk, including the chunk type code and chunk data fields, but not
    /// including the length field. The CRC is always present, even for chunks
    /// containing no data.
    crc: u32,
}

// Allow dead code since some functions are only called in tests
#[allow(dead_code)]
impl Chunk {
    fn length(&self) -> u32 {
        self.length
    }

    fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    fn data(&self) -> &[u8] {
        &self.chunk_data
    }

    fn crc(&self) -> u32 {
        self.crc
    }

    fn data_as_string(&self) -> crate::Result<String> {
        Ok(String::from_utf8(self.chunk_data.clone()).map_err(Box::new)?)
    }
}

#[derive(Debug)]
pub struct BadChunkError {
    /// Why is it a bad chunk?
    reason: String,
}
impl BadChunkError {
    fn boxed(reason: String) -> Box<Self> {
        Box::new(Self { reason })
    }
}

impl fmt::Display for BadChunkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bad chunk: {}", self.reason)
    }
}
impl Error for BadChunkError {}

impl TryFrom<&[u8]> for Chunk {
    type Error = crate::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        // Track where we are in the chunk
        let mut position = 0;
        let length: u32 = u32::from_be_bytes(bytes[position..position + 4].try_into()?);
        position += 4;
        if length > MAXIMUM_LENGTH {
            return Err(BadChunkError::boxed(format!(
                "Length is too long ({} > 2^31 - 1)",
                length
            )));
        }
        let type_bytes: &[u8; 4] = &bytes[position..position + 4].try_into()?;
        position += 4;
        let chunk_type: ChunkType = ChunkType::try_from(*type_bytes)?;
        let chunk_data: Vec<u8> =
            bytes[position..position + usize::try_from(length)?].try_into()?;
        position += usize::try_from(length)?;
        if chunk_data.len() != length.try_into()? {
            return Err(BadChunkError::boxed(format!(
                "Data (len {}) is the wrong length (expected {})",
                chunk_data.len(),
                length
            )));
        }
        let provided_crc = u32::from_be_bytes(bytes[position..position + 4].try_into()?);
        let true_crc =
            crc::crc32::checksum_ieee(&[&chunk_type.bytes(), chunk_data.as_slice()].concat());
        if provided_crc != true_crc {
            return Err(BadChunkError::boxed(format!(
                "Bad CRC (received {}, expected {})",
                provided_crc, true_crc
            )));
        }
        Ok(Chunk {
            length,
            chunk_type,
            chunk_data,
            crc: true_crc,
        })
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "len: {}, type:  {}, data: {}",
            self.length(),
            self.chunk_type(),
            self.data_as_string()
                .unwrap_or_else(|_| "[invalid data]".to_string())
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = b"RuSt";
        let message_bytes = b"This is where your secret message will be!";
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = b"RuSt";
        let message_bytes = b"This is where your secret message will be!";
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = b"RuSt";
        let message_bytes = b"This is where your secret message will be!";
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = b"RuSt";
        let message_bytes = b"This is where your secret message will be!";
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
