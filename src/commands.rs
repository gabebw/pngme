use crate::args::Subcommand;
use crate::chunk::Chunk;
use crate::png::Png;
use std::convert::TryFrom;
use std::fs;

pub fn run(subcommand: Subcommand) -> crate::Result<()> {
    match subcommand {
        Subcommand::Encode {
            input_file_path,
            chunk_type,
            message,
            output_file_path,
        } => {
            let input_bytes = fs::read(&input_file_path)?;
            let output = output_file_path.unwrap_or(input_file_path);
            let mut png = Png::try_from(input_bytes.as_slice())?;
            let chunk = Chunk::new(chunk_type, message.as_bytes().to_vec());
            png.append_chunk(chunk);
            fs::write(output, png.as_bytes())?;
        }
        Subcommand::Decode {
            file_path,
            chunk_type,
        } => {
            let input_bytes = fs::read(&file_path)?;
            let png = Png::try_from(input_bytes.as_slice())?;
            let chunk = png.chunk_by_type(chunk_type);
            if let Some(c) = chunk {
                println!("{}", c);
            }
        }
        Subcommand::Remove {
            file_path,
            chunk_type,
        } => {
            let input_bytes = fs::read(&file_path)?;
            let mut png = Png::try_from(input_bytes.as_slice())?;
            match png.remove_chunk(chunk_type) {
                Ok(chunk) => {
                    fs::write(&file_path, png.as_bytes())?;
                    println!("Removed chunk: {}", chunk);
                }
                Err(e) => println!("Error: {}", e),
            }
        }
        Subcommand::Print { file_path } => {
            let input_bytes = fs::read(&file_path)?;
            let png = Png::try_from(input_bytes.as_slice())?;
            for chunk in png.chunks() {
                println!("{}", chunk);
            }
        }
    }
    Ok(())
}
