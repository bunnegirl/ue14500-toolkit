use std::fmt::{Binary, Octal, Formatter, Result as FmtResult};

#[derive(Debug, Clone, PartialEq)]
pub struct Words(pub Vec<Word>);

#[derive(Debug, Clone, PartialEq)]
pub struct Word(pub Inst, pub Addr, pub Ctrl);

/// represents a 12-bit code word for the ue14500 processor
impl Word {
    /// get the inst from the word
    pub fn inst(&self) -> Inst {
        self.0
    }

    /// create a new word with the provided addr
    pub fn with_inst(self, inst: Inst) -> Word {
        Word(inst, self.1, self.2)
    }

    /// get the addr from the word
    pub fn addr(&self) -> Addr {
        self.1
    }

    /// create a new word with the provided addr
    pub fn with_addr(self, addr: Addr) -> Word {
        Word(self.0, addr, self.2)
    }

    /// get the ctrl from the word
    pub fn ctrl(&self) -> Ctrl {
        self.2
    }

    /// create a new word with the provided ctrl
    pub fn with_ctrl(self, ctrl: Ctrl) -> Word {
        Word(self.0, self.1, ctrl)
    }
}

impl From<u32> for Word {
    fn from(bin: u32) -> Word {
        let inst = Inst::from(bin);
        let addr = Addr::from(bin);
        let ctrl = Ctrl::from(bin);

        Word(inst, addr, ctrl)
    }
}

impl From<Word> for u32 {
    fn from(word: Word) -> u32 {
        let Word(inst, addr, ctrl) = word;

        inst.code() << INST_POS | addr.0 << ADDR_POS | ctrl.0
    }
}

pub const INST_MASK: u32 = 0b1111_000000_00;
pub const INST_POS: u32 = 8;
pub const INST_TABLE: [(u32, &str, Inst); 16] = [
    (0b0000, "nop0", Inst::Nop0),
    (0b0001, "ld", Inst::Ld),
    (0b0010, "add", Inst::Add),
    (0b0011, "sub", Inst::Sub),
    (0b0100, "one", Inst::One),
    (0b0101, "nand", Inst::Nand),
    (0b0110, "or", Inst::Or),
    (0b0111, "xor", Inst::Xor),
    (0b1000, "sto", Inst::Sto),
    (0b1001, "stoc", Inst::StoC),
    (0b1010, "ien", Inst::Ien),
    (0b1011, "oen", Inst::Oen),
    (0b1100, "ioc", Inst::Ioc),
    (0b1101, "rtn", Inst::Rtn),
    (0b1110, "skz", Inst::Skz),
    (0b1111, "nopf", Inst::NopF),
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Inst {
    Nop0,
    Ld,
    Add,
    Sub,
    One,
    Nand,
    Or,
    Xor,
    Sto,
    StoC,
    Ien,
    Oen,
    Ioc,
    Rtn,
    Skz,
    NopF,
}

impl Inst {
    pub fn code(self) -> u32 {
        INST_TABLE[self as usize].0
    }

    pub fn name(self) -> &'static str {
        INST_TABLE[self as usize].1
    }
}

impl Binary for Inst {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{:04b}", self.code())
    }
}

impl Octal for Inst {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{:02o}", self.code())
    }
}

impl From<u8> for Inst {
    fn from(word: u8) -> Inst {
        Inst::from(word as u32)
    }
}

impl From<u32> for Inst {
    fn from(word: u32) -> Inst {
        let lookup = (word & INST_MASK) >> INST_POS;

        for (code, _, inst) in INST_TABLE {
            if code == lookup {
                return inst;
            }
        }

        unreachable!();
    }
}

pub const ADDR_MASK: u32 = 0b0000_111111_00;
pub const ADDR_POS: u32 = 2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Addr(pub u32);

impl Addr {
    pub fn bits(&self) -> u32 {
        self.0
    }
}

impl Binary for Addr {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{:06b}", self.0)
    }
}

impl Octal for Addr {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{:02o}", self.0)
    }
}

impl From<u8> for Addr {
    fn from(word: u8) -> Addr {
        Addr::from(word as u32)
    }
}

impl From<u32> for Addr {
    fn from(word: u32) -> Addr {
        Addr((word & ADDR_MASK) >> ADDR_POS)
    }
}

const CTRL_MASK: u32 = 0b0000_000000_11;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ctrl(pub u32);

impl Ctrl {
    pub fn bits(&self) -> u32 {
        self.0
    }
}

impl Binary for Ctrl {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{:02b}", self.0)
    }
}

impl Octal for Ctrl {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{:01o}", self.0)
    }
}

impl From<u8> for Ctrl {
    fn from(word: u8) -> Ctrl {
        Ctrl::from(word as u32)
    }
}

impl From<u32> for Ctrl {
    fn from(word: u32) -> Ctrl {
        Ctrl(word & CTRL_MASK)
    }
}
