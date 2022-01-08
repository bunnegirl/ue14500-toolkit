pub mod parser;

use crate::data::{Words};
use chonk::framework::{Parser, ParserResultMapper};
use std::fs::File;
use std::io::{prelude::*, Result};
use std::path::PathBuf;

/// read a file from disk and deserialise words from binary
pub fn read_file(path: PathBuf) -> Result<Words> {
    let mut buffer = File::open(path).expect("error opening file for reading");
    let mut asm = String::new();

    buffer.read_to_string(&mut asm).unwrap();

    let words = parser::words().parse(&asm).unwrap_result();
    
    Ok(words)
}
