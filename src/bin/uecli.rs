#![allow(clippy::unusual_byte_groupings)]
#![allow(dead_code)]

use clap::{ArgEnum, Subcommand, Parser};
use core::num;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use ue14500_toolkit::{
    data::Words,
    formats::{assembly, binary},
};

const ASM_EXTENSION: &str = "asm";
const BIN_EXTENSION: &str = "bin";

#[derive(Parser, Debug, PartialEq)]
#[clap(about, author, version)]
/// Cli tools for the Usagi Electric ue14500 processor
struct Opt {
    /// Number format
    #[clap(long, short = 'n')]
    #[clap(arg_enum, default_value = "bin")]
    numbers: NumberFormat,
    #[clap(subcommand)]
    command: Cmd,
}

#[derive(Subcommand, Debug, PartialEq)]
enum Cmd {
    /// Assemble binary
    Asm {
        /// Assembly input
        #[clap(parse(try_from_str))]
        from: InputPath,
        /// Binary output
        #[clap(parse(try_from_str))]
        into: OutputPath,
    },

    /// Disassemble binary
    Dsm {
        /// Binary input
        #[clap(parse(try_from_str))]
        from: InputPath,
        /// Assembly output
        #[clap(parse(try_from_str))]
        into: OutputPath,
    },

    /// List binary contents
    List {
        /// Binary input
        #[clap(parse(try_from_str))]
        from: InputPath,
    },
}

#[derive(ArgEnum, Clone, Debug, PartialEq)]
pub enum NumberFormat {
    Bin,
    Oct,
}

impl Default for NumberFormat {
    fn default() -> Self {
        NumberFormat::Bin
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
    let Opt { numbers, command } = Opt::parse();

    match command {
        Cmd::Asm { from, into } => asm(numbers, from.into(), into.into()),
        Cmd::Dsm { from, into } => asm(numbers, from.into(), into.into()),
        Cmd::List { from } => list(numbers, from.into())
    }
}

fn asm(numbers: NumberFormat, from: PathBuf, into: PathBuf) {
    binary::write_file(
        into.clone(),
        assembly::read_file(from)
            .expect("error reading assembly"),
    )
    .expect("error writing binary");

    list(numbers, into);
}

fn dsm(numbers: NumberFormat, from: PathBuf, into: PathBuf) {
    println!("disassembly not yet implemented")
}

fn list(numbers: NumberFormat, from: PathBuf) {
    use prettytable::{
        cell,
        format::{FormatBuilder, LinePosition, LineSeparator},
        row, Table,
    };
    use NumberFormat::*;

    let Words(words) = binary::read_file(from)
        .expect("error reading binary");

    let mut table = Table::new();

    table.set_format(
        FormatBuilder::new()
            .column_separator('│')
            .borders('┃')
            .separators(
                &[LinePosition::Title],
                LineSeparator::new('─', '┼', '┠', '┨'),
            )
            .separators(&[LinePosition::Top], LineSeparator::new('─', '┬', '┎', '┒'))
            .separators(
                &[LinePosition::Bottom],
                LineSeparator::new('─', '┴', '┖', '┚'),
            )
            .padding(1, 1)
            .indent(2)
            .build(),
    );

    table.set_titles(row!["#", "Instruction", "Address", "I/O Control"]);

    for (index, word) in words.iter().enumerate() {
        let inst = match numbers {
            Bin => format!("0b{:b}", word.inst()),
            Oct => format!("0o{:o}", word.inst()),
        };
        let addr = match numbers {
            Bin => format!("0b{:b}", word.addr()),
            Oct => format!("0o{:o}", word.addr()),
        };
        let ctrl = match numbers {
            Bin => format!("0b{:b}", word.ctrl()),
            Oct => format!("0o{:o}", word.ctrl()),
        };

        table.add_row(row![index, inst, addr, ctrl]);
    }

    table.printstd();
}
