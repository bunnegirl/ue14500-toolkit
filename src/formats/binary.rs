use crate::data::{Nodes, *};
use bitbit::{BitReader, BitWriter};
use std::fs::File;
use std::io::{prelude::*, BufReader, BufWriter, Result};
use std::path::PathBuf;

/// read a file from disk and deserialise words from binary
pub fn read_file(path: PathBuf) -> Result<Nodes> {
    let mut buffer = File::open(path).expect("error opening file for reading");

    deserialize(&mut buffer)
}

/// deserialize words from binary with any reader
pub fn deserialize(input: &mut impl Read) -> Result<Nodes> {
    let buf = BufReader::new(input);
    let mut bitreader: BitReader<_, bitbit::MSB> = BitReader::new(buf);
    let mut nodes = Vec::new();

    loop {
        match bitreader.read_bits(12) {
            Err(_) => break,
            Ok(bits) => nodes.push(Word::from(bits).into()),
        }
    }

    Ok(Nodes(nodes))
}

/// serialize words to binary and write a file to disk
pub fn write_file(path: PathBuf, words: Nodes) -> Result<()> {
    let mut buffer =
        File::create(path).expect("error opening file for writing");

    serialize(&mut buffer, words)
}

/// serialize nodes to binary with any writer
pub fn serialize(output: &mut impl Write, nodes: Nodes) -> Result<()> {
    let mut buf = BufWriter::new(output);
    let mut bitwriter = BitWriter::new(&mut buf);
    let Nodes(nodes) = nodes;

    for node in nodes {
        if let Ok(word) = TryInto::<Word>::try_into(node) {
            bitwriter.write_bits(word.inst().val(), 4)?;
            bitwriter.write_bits(word.addr().val(), 6)?;
            bitwriter.write_bits(word.ctrl().val(), 2)?;
        }
    }

    bitwriter.pad_to_byte()?;
    buf.flush()?;

    Ok(())
}
