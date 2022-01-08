#![allow(clippy::unusual_byte_groupings)]
#![allow(dead_code)]

use clap::Parser;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use ue14500_toolkit::{
    data::{Word, Words},
    formats::{assembly, binary},
};

const ASM_EXTENSION: &str = "asm";
const BIN_EXTENSION: &str = "bin";

#[derive(Parser, Debug, PartialEq)]
#[clap(about, author, version)]
/// cli tools for the usagi electric ue14500 processor
enum Opt {
    /// assemble binary
    Asm {
        #[clap(parse(try_from_str))]
        from: InputPath,
        #[clap(parse(try_from_str))]
        into: OutputPath,
    },

    /// disassemble binary
    Dsm {
        #[clap(parse(try_from_str))]
        from: InputPath,
        #[clap(parse(try_from_str))]
        into: OutputPath,
    },

    /// list binary contents
    List {
        #[clap(parse(try_from_str))]
        from: InputPath,
    }
}

#[derive(Debug, PartialEq)]
pub struct InputPath(pub PathBuf);

impl From<InputPath> for PathBuf {
    fn from(path: InputPath) -> PathBuf {
        path.0
    }
}

impl FromStr for InputPath {
    type Err = String;

    fn from_str(val: &str) -> Result<InputPath, Self::Err> {
        match validate_file(val) {
            Ok(path) => match validate_file_readable(&path) {
                Ok(_) => Ok(InputPath(path)),
                Err(err) => Err(err),
            },
            Err(err) => Err(err),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct OutputPath(pub PathBuf);

impl From<OutputPath> for PathBuf {
    fn from(path: OutputPath) -> PathBuf {
        path.0
    }
}

impl FromStr for OutputPath {
    type Err = String;

    fn from_str(val: &str) -> Result<OutputPath, Self::Err> {
        match validate_file(val) {
            Ok(path) => match validate_file_writable(&path) {
                Ok(_) => Ok(OutputPath(path)),
                Err(err) => Err(err),
            },
            Err(_) => Err("invalid path".into()),
        }
    }
}

fn validate_file(val: &str) -> Result<PathBuf, String> {
    match PathBuf::from_str(val) {
        Ok(path) => Ok(path),
        Err(_) => Err("invalid path".into()),
    }
}

fn validate_file_readable(path: &Path) -> Result<(), String> {
    match path.is_file() {
        true => Ok(()),
        false => Err("expected a file ".into()),
    }
}

fn validate_file_writable(path: &Path) -> Result<(), String> {
    match path.metadata() {
        Ok(meta) => match (!meta.permissions().readonly(), !path.is_dir()) {
            (true, true) => Ok(()),
            (false, true) => Err("file is readonly".into()),
            _ => Err("expected a file and found a directory".into()),
        },
        Err(_) => Err("could not read file metadata".into()),
    }
}

fn main() {
    match Opt::parse() {
        Opt::Asm { from, into } => {
            let words = assembly::read_file(from.into()).expect("error reading assembly");

            pretty_print(&words);

            binary::write_file(into.into(), words).expect("error writing binary");
        }
        _ => unimplemented!(),
    }
}

fn pretty_print(words: &Words) {
    let Words(words) = words;

    println!("  inst  addr    ctrl");

    for word in words {
        let Word(inst, addr, ctrl) = word;

        println!("  {:b}  {:b}  {:b}", inst, addr, ctrl);
    }
}
