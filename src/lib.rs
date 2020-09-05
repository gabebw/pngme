#[doc(inline)]
mod chunk;
#[doc(inline)]
mod chunk_type;
#[doc(inline)]
mod png;

#[doc(inline)]
pub use chunk::Chunk;
#[doc(inline)]
pub use chunk_type::ChunkType;
#[doc(inline)]
pub use png::Png;

/// Holds any kind of error.
pub type Error = Box<dyn std::error::Error>;
/// Holds a `Result` of any kind of error.
pub type Result<T> = std::result::Result<T, Error>;
