#![allow(clippy::unusual_byte_groupings)]
#![allow(dead_code)]

use ue14500_toolkit::{
    data::{Words, Word},
    formats::{assembly, binary}
};
use std::path::PathBuf;
use clap::*;

#[derive(Parser, Debug, PartialEq)]
#[clap(about, author, version)]
enum Opt {
    /// assemble binary
    Asm {
        from: PathBuf,
        into: PathBuf,
    },
    /// dissemble binary
    Dsm {
        from: PathBuf,
        into: PathBuf,
    }
}

fn main() {
    match Opt::parse() {
        // assemble binary
        Opt::Asm { from, into } => {
            let words = assembly::read_file(from).expect("error reading assembly");

            pretty_print(&words);

            binary::write_file(into, words).expect("error writing binary");
        },
        _ => unimplemented!()
    }
}

fn pretty_print(words: &Words) {
    println!("  inst  addr    ctrl");
    let Words(words) = words;

    for word in words {
        let Word(inst, addr, ctrl) = word;

        println!("  {:b}  {:b}  {:b}", inst, addr, ctrl);
    }
}