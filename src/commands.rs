use crate::args::*;
use crate::chunk::Chunk;
use crate::png::Png;
use std::convert::TryFrom;
use std::fs;

fn encode(args: EncodeArgs) -> crate::Result<()> {
    let input_bytes = fs::read(&args.input_file_path)?;
    let output = args.output_file_path.unwrap_or(args.input_file_path);
    let mut png = Png::try_from(input_bytes.as_slice())?;
    let chunk = Chunk::new(args.chunk_type, args.message.as_bytes().to_vec());
    png.append_chunk(chunk);
    fs::write(output, png.as_bytes())?;
    Ok(())
}

fn decode(args: DecodeArgs) -> crate::Result<()> {
    let input_bytes = fs::read(&args.file_path)?;
    let png = Png::try_from(input_bytes.as_slice())?;
    let chunk = png.chunk_by_type(args.chunk_type);
    if let Some(c) = chunk {
        println!("{}", c);
    }
    Ok(())
}

fn remove(args: RemoveArgs) -> crate::Result<()> {
    let input_bytes = fs::read(&args.file_path)?;
    let mut png = Png::try_from(input_bytes.as_slice())?;
    match png.remove_chunk(args.chunk_type) {
        Ok(chunk) => {
            fs::write(&args.file_path, png.as_bytes())?;
            println!("Removed chunk: {}", chunk);
        }
        Err(e) => println!("Error: {}", e),
    }
    Ok(())
}

fn print(args: PrintArgs) -> crate::Result<()> {
    let input_bytes = fs::read(&args.file_path)?;
    let png = Png::try_from(input_bytes.as_slice())?;
    for chunk in png.chunks() {
        println!("{}", chunk);
    }
    Ok(())
}

pub fn run(subcommand: Subcommand) -> crate::Result<()> {
    match subcommand {
        Subcommand::Encode(args) => encode(args),
        Subcommand::Decode(args) => decode(args),
        Subcommand::Remove(args) => remove(args),
        Subcommand::Print(args) => print(args),
    }
}
