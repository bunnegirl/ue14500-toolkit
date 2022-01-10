#![allow(clippy::unusual_byte_groupings)]
#![allow(dead_code)]

use clap::{ArgEnum, Parser, Subcommand};
use comfy_table::{presets::NOTHING, *};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use ue14500_toolkit::{
    data::{Node, Nodes},
    formats::{assembly, binary, FileType},
};

const TABLE_STYLE: &str = "││──├─┼┤│    ┬┴╭╮╰╯";

/// Command line tools for the Usagi Electric ue14500 processor
#[derive(Parser, Debug, PartialEq)]
#[clap(name = "uecli")]
#[clap(version = "0.1")]
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
        /// List file contents
        #[clap(long, short = 'l')]
        list: bool,
        /// Assembly input
        #[clap(parse(try_from_str))]
        from: InputPath,
        /// Binary output
        #[clap(parse(try_from_str))]
        into: OutputPath,
    },

    /// Disassemble binary
    Dsm {
        /// List file contents
        #[clap(long, short = 'l')]
        list: bool,
        /// Binary input
        #[clap(parse(try_from_str))]
        from: InputPath,
        /// Assembly output
        #[clap(parse(try_from_str))]
        into: OutputPath,
    },

    /// List file contents
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
        Cmd::Asm {
            list,
            from: InputPath(from),
            into: OutputPath(into),
        } => {
            run_asm(from, into.clone());

            if list {
                run_list(numbers, into)
            }
        }
        Cmd::Dsm {
            list,
            from: InputPath(from),
            into: OutputPath(into),
        } => {
            run_dsm(from, into.clone());

            if list {
                run_list(numbers, into)
            }
        }
        Cmd::List {
            from: InputPath(from),
        } => run_list(numbers, from),
    }
}

fn run_asm(from: PathBuf, into: PathBuf) {
    binary::write_file(
        into,
        assembly::read_file(from).expect("error reading assembly"),
    )
    .expect("error writing binary");
}

fn run_dsm(_from: PathBuf, _into: PathBuf) {
    println!("disassembly not yet implemented")
}

fn run_list(numbers: NumberFormat, from: PathBuf) {
    use NumberFormat::*;

    let Nodes(nodes) = match FileType::try_from(from.clone())
        .expect("assembly or binary file")
    {
        FileType::Assembly => {
            assembly::read_file(from).expect("error reading assembly")
        }
        FileType::Binary => {
            binary::read_file(from).expect("error reading binary")
        }
    };

    let mut tables = Vec::new();
    let mut table = new_list_table();

    table.set_header(vec!["#", "Instruction", "Address", "I/O Control"]);

    let mut words = 0;
    let mut is_comment = false;
    let mut indent = 0;

    for node in nodes {
        match node {
            Node::Word(inst, addr, ctrl) => {
                if is_comment {
                    tables.push((is_comment, indent, table));
                    table = new_list_table();
                    is_comment = false;
                    indent = 0;
                }

                let inst = match numbers {
                    Bin => format!("0b{:b}{:>6}", inst, inst.name()),
                    Oct => format!("0o{:o}{:>7}", inst, inst.name()),
                };

                let addr = match numbers {
                    Bin => format!("0b{:b}{:>20}", addr, addr.name()),
                    Oct => format!("0o{:o}{:>20}", addr, addr.name()),
                };

                let ctrl = match numbers {
                    Bin => format!("0b{:b}{:>20}", ctrl, ctrl.name()),
                    Oct => format!("0o{:o}{:>20}", ctrl, ctrl.name()),
                };

                table.add_row(vec![format!("{}", words), inst, addr, ctrl]);

                words += 1;
            }
            Node::Comment(text) => {
                if !is_comment {
                    tables.push((is_comment, indent, table));
                    table = new_clean_table();
                    is_comment = true;
                    indent = usize::MAX;
                }

                // find the minimum indentation of non empty lines
                if !text.trim().is_empty() {
                    let line_indent = text.len() - text.trim_start().len();

                    if line_indent < indent {
                        indent = line_indent;
                    }
                }

                table.add_row(vec!["", &text]);
            }
        }
    }

    tables.push((is_comment, indent, table));

    for (is_comment, indent, table) in tables {
        print_table(is_comment, words, indent, table);
    }
}

fn new_list_table() -> Table {
    let mut table = Table::new();

    table.load_preset(TABLE_STYLE);

    table
}

fn new_clean_table() -> Table {
    let mut table = Table::new();

    table.load_preset(NOTHING);

    table
}

fn print_table(
    is_comment: bool,
    words: usize,
    indent: usize,
    mut table: Table,
) {
    let column = table.get_column_mut(0).expect("first column");

    column.set_cell_alignment(CellAlignment::Right);
    column.set_constraint(ColumnConstraint::Absolute(Width::Fixed(
        2 + (format!("{}", words).len()) as u16,
    )));

    if is_comment && indent > 0 {
        for line in table.lines() {
            if line.len() > indent {
                println!("  {}", &line[indent..]);
            } else {
                println!("  {}", line);
            }
        }
    } else {
        println!("{}", table);
    }
}
