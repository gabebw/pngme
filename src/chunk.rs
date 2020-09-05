use crate::chunk_type::ChunkType;
use std::convert::{TryFrom, TryInto};
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::io::{BufReader, Read};

const MAXIMUM_LENGTH: u32 = (1 << 31) - 1;

/// Each chunk consists of four parts: length, chunk type, chunk data, and CRC.
pub struct Chunk {
    /// A 4-byte unsigned integer giving the number of bytes in the chunk's data
    /// field. The length counts *only* the data field, *not* itself, the chunk
    /// type code, or the CRC. Zero is a valid length. Although encoders and
    /// decoders should treat the length as unsigned, its value must not exceed
    /// 2^31-1 bytes.
    length: u32,

    /// The chunk type.
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

impl Chunk {
    /// Build a chunk from a [ChunkType](../chunk_type/struct.ChunkType.html) and
    /// chunk data.
    pub fn new(chunk_type: ChunkType, chunk_data: Vec<u8>) -> Self {
        let crc = crc::crc32::checksum_ieee(&[&chunk_type.bytes(), chunk_data.as_slice()].concat());
        Chunk {
            length: chunk_data.len() as u32,
            chunk_type,
            chunk_data,
            crc,
        }
    }

    /// The length field. Note that this is *not* the total number of bytes in the
    /// Chunk; it is the length of the `chunk.data()`.
    /// To get the total number of bytes in the chunk, call
    /// `chunk.as_bytes().len()`.
    pub fn length(&self) -> u32 {
        self.length
    }

    /// The chunk type.
    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    /// The chunk data.
    fn data(&self) -> &[u8] {
        &self.chunk_data
    }

    /// The pre-calculated CRC (cyclic redundancy check).
    fn crc(&self) -> u32 {
        self.crc
    }

    /// Attempt to represent the data a UTF-8 string. Returns `Err` if it could
    /// not decode to a String.
    pub fn data_as_string(&self) -> crate::Result<String> {
        Ok(String::from_utf8(self.chunk_data.clone()).map_err(Box::new)?)
    }

    /// Every byte in this chunk.
    pub fn as_bytes(&self) -> Vec<u8> {
        self.length()
            .to_be_bytes()
            .iter()
            .chain(self.chunk_type().bytes().iter())
            .chain(self.data().iter())
            .chain(self.crc().to_be_bytes().iter())
            .copied()
            .collect::<Vec<u8>>()
    }
}

/// Something went wrong while decoding a chunk.
#[derive(Debug)]
pub struct ChunkDecodingError {
    /// The reason that decoding went wrong.
    reason: String,
}
impl ChunkDecodingError {
    fn boxed(reason: String) -> Box<Self> {
        Box::new(Self { reason })
    }
}

impl fmt::Display for ChunkDecodingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bad chunk: {}", self.reason)
    }
}
impl Error for ChunkDecodingError {}

impl TryFrom<&[u8]> for Chunk {
    type Error = crate::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let mut reader = BufReader::new(bytes);
        // Store the various 4-byte values in a chunk
        let mut buf: [u8; 4] = [0; 4];
        reader.read_exact(&mut buf)?;
        let length = u32::from_be_bytes(buf);
        if length > MAXIMUM_LENGTH {
            return Err(ChunkDecodingError::boxed(format!(
                "Length is too long ({} > 2^31 - 1)",
                length
            )));
        }
        reader.read_exact(&mut buf)?;
        let chunk_type: ChunkType = ChunkType::try_from(buf)?;
        let mut chunk_data: Vec<u8> = vec![0; usize::try_from(length)?];
        reader.read_exact(&mut chunk_data)?;
        if chunk_data.len() != length.try_into()? {
            return Err(ChunkDecodingError::boxed(format!(
                "Data (len {}) is the wrong length (expected {})",
                chunk_data.len(),
                length
            )));
        }
        reader.read_exact(&mut buf)?;
        let provided_crc = u32::from_be_bytes(buf);
        let true_crc =
            crc::crc32::checksum_ieee(&[&chunk_type.bytes(), chunk_data.as_slice()].concat());
        if provided_crc != true_crc {
            return Err(ChunkDecodingError::boxed(format!(
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
            "{}\t{}",
            self.chunk_type(),
            self.data_as_string()
                .unwrap_or_else(|_| "[data]".to_string())
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
