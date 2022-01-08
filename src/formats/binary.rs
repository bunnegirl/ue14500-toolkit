use crate::data::{Words, *};
use bitbit::{BitReader, BitWriter};
use std::fs::File;
use std::io::{prelude::*, Result};
use std::path::PathBuf;

/// read a file from disk and deserialise words from binary
pub fn read_file(path: PathBuf) -> Result<Words> {
    let mut buffer = File::open(path).expect("error opening file for reading");

    deserialize(&mut buffer)
}

/// deserialize words from binary with any reader
pub fn deserialize(input: &mut impl Read) -> Result<Words> {
    let mut bitreader: BitReader<_, bitbit::MSB> = BitReader::new(input);
    let mut words = Vec::new();

    loop {
        match bitreader.read_bits(12) {
            Err(_) => break,
            Ok(buf) => {
                words.push(Word::from(buf))
            },
        }
    }

    Ok(Words(words))
}

/// serialize words to binary and write a file to disk
pub fn write_file(path: PathBuf, words: Words) -> Result<()> {
    let mut buffer = File::create(path).expect("error opening file for writing");

    serialize(&mut buffer, words)
}

/// serialize words to binary with any writer
pub fn serialize(output: &mut impl Write, words: Words) -> Result<()> {
    let mut bitwriter = BitWriter::new(output);
    let Words(words) = words;

    for word in words {
        bitwriter.write_bits(word.inst().code(), 4).unwrap();
        bitwriter.write_bits(word.addr().bits(), 6).unwrap();
        bitwriter.write_bits(word.ctrl().bits(), 2).unwrap();
    }

    Ok(())
}
