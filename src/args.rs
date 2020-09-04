use crate::chunk_type::ChunkType;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::clap::AppSettings;
use structopt::StructOpt;

/*
pngme encode ./dice.png ruSt "This is a secret message!
pngme decode ./dice.png ruSt
pngme remove ./dice.png ruSt
pngme print ./dice.png
*/

#[derive(StructOpt)]
#[structopt(global_settings(&[AppSettings::VersionlessSubcommands]))]
pub struct Cli {
    #[structopt(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(Debug, StructOpt, PartialEq)]
pub enum Subcommand {
    #[structopt(about = "Add a secret message to a PNG")]
    Encode {
        #[structopt(parse(from_os_str), help = "Path to the input PNG")]
        input_file_path: PathBuf,
        #[structopt(
            parse(try_from_str = ChunkType::from_str),
            help = "Chunk type (like 'ruSt')"
        )]
        chunk_type: ChunkType,
        #[structopt(help = "Your secret message")]
        message: String,
        #[structopt(parse(from_os_str), help = "Path to the output PNG (optional)")]
        output_file_path: Option<PathBuf>,
    },
    #[structopt(about = "Show the secret message in a PNG")]
    Decode {
        #[structopt(parse(from_os_str), help = "Path to the PNG")]
        file_path: PathBuf,
        #[structopt(
            parse(try_from_str = ChunkType::from_str),
            help = "Chunk type (like 'ruSt')"
        )]
        chunk_type: ChunkType,
    },
    #[structopt(about = "Remove a secret message from a PNG")]
    Remove {
        #[structopt(parse(from_os_str), help = "Path to the PNG")]
        file_path: PathBuf,
        #[structopt(
            parse(try_from_str = ChunkType::from_str),
            help = "Chunk type (like 'ruSt')"
        )]
        chunk_type: ChunkType,
    },
    #[structopt(about = "Print every chunk in a PNG")]
    Print {
        #[structopt(parse(from_os_str), help = "Path to the PNG")]
        file_path: PathBuf,
    },
}

mod test {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    pub fn test_encode() {
        let expected = Subcommand::Encode {
            input_file_path: PathBuf::from("/a/b/c"),
            chunk_type: ChunkType::from_str("RuSt").unwrap(),
            message: "Secret decoder ring".to_string(),
            output_file_path: None,
        };
        let cli = Cli::from_iter(vec![
            "pngme",
            "encode",
            "/a/b/c",
            "RuSt",
            "Secret decoder ring",
        ]);
        let actual = cli.subcommand;

        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_encode_with_output_file() {
        let expected = Subcommand::Encode {
            input_file_path: PathBuf::from("/a/b/c"),
            chunk_type: ChunkType::from_str("RuSt").unwrap(),
            message: "Secret decoder ring".to_string(),
            output_file_path: Some(PathBuf::from("/output/file/path")),
        };
        let cli = Cli::from_iter(vec![
            "pngme",
            "encode",
            "/a/b/c",
            "RuSt",
            "Secret decoder ring",
            "/output/file/path",
        ]);
        let actual = cli.subcommand;

        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_decode() {
        let expected = Subcommand::Decode {
            file_path: PathBuf::from("/a/b/c"),
            chunk_type: ChunkType::from_str("PnGm").unwrap(),
        };
        let cli = Cli::from_iter(vec!["pngme", "decode", "/a/b/c", "PnGm"]);
        let actual = cli.subcommand;

        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_remove() {
        let expected = Subcommand::Remove {
            file_path: PathBuf::from("/a/b/c"),
            chunk_type: ChunkType::from_str("imAG").unwrap(),
        };
        let cli = Cli::from_iter(vec!["pngme", "remove", "/a/b/c", "imAG"]);
        let actual = cli.subcommand;

        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_print() {
        let expected = Subcommand::Print {
            file_path: PathBuf::from("/a/b/c"),
        };
        let cli = Cli::from_iter(vec!["pngme", "print", "/a/b/c"]);
        let actual = cli.subcommand;

        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_unknown_subcommand() {
        let result = Cli::from_iter_safe(vec!["pngme", "blah-blah", "some-argument"]);

        assert!(result.is_err());
    }
}
