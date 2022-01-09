use std::fmt::{Binary, Display, Formatter, Octal, Result as FmtResult};
use std::ops::RangeInclusive;

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

        inst.val() << INST_POS | addr.val() << ADDR_POS | ctrl.val()
    }
}

pub const INST_MASK: u32 = 0b1111_000000_00;
pub const INST_POS: u32 = 8;
pub const INST_TABLE: [(u32, &str, InstKind); 16] = [
    (
        //
        0b0000,
        "nop0",
        InstKind::Nop0,
    ),
    (
        //
        0b0001,
        "ld",
        InstKind::Ld,
    ),
    (
        //
        0b0010,
        "add",
        InstKind::Add,
    ),
    (
        //
        0b0011,
        "sub",
        InstKind::Sub,
    ),
    (
        //
        0b0100,
        "one",
        InstKind::One,
    ),
    (
        //
        0b0101,
        "nand",
        InstKind::Nand,
    ),
    (
        //
        0b0110,
        "or",
        InstKind::Or,
    ),
    (
        //
        0b0111,
        "xor",
        InstKind::Xor,
    ),
    (
        //
        0b1000,
        "sto",
        InstKind::Sto,
    ),
    (
        //
        0b1001,
        "stoc",
        InstKind::StoC,
    ),
    (
        //
        0b1010,
        "ien",
        InstKind::Ien,
    ),
    (
        //
        0b1011,
        "oen",
        InstKind::Oen,
    ),
    (
        //
        0b1100,
        "ioc",
        InstKind::Ioc,
    ),
    (
        //
        0b1101,
        "rtn",
        InstKind::Rtn,
    ),
    (
        //
        0b1110,
        "skz",
        InstKind::Skz,
    ),
    (
        //
        0b1111,
        "nopf",
        InstKind::NopF,
    ),
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InstKind {
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

impl InstKind {
    pub fn name(self) -> &'static str {
        INST_TABLE[self as usize].1
    }

    pub fn val(self) -> u32 {
        INST_TABLE[self as usize].0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Inst(InstKind);

impl Inst {
    pub fn kind(self) -> InstKind {
        self.0
    }

    pub fn name(self) -> &'static str {
        self.kind().name()
    }

    pub fn val(self) -> u32 {
        self.kind().val()
    }
}

impl Binary for Inst {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{:04b}", self.val())
    }
}

impl Octal for Inst {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{:02o}", self.val())
    }
}

impl From<InstKind> for Inst {
    fn from(kind: InstKind) -> Inst {
        Inst(kind)
    }
}

impl From<u8> for Inst {
    fn from(word: u8) -> Inst {
        Inst::from(word as u32)
    }
}

impl From<u32> for Inst {
    fn from(word: u32) -> Inst {
        let val = (word & INST_MASK) >> INST_POS;

        for (tbl_val, _, tbl_kind) in INST_TABLE {
            if tbl_val == val {
                return Inst(tbl_kind);
            }
        }

        unreachable!();
    }
}

pub const ADDR_MASK: u32 = 0b0000_111111_00;
pub const ADDR_POS: u32 = 2;
pub const ADDR_TABLE: [(RangeInclusive<u32>, &str, AddrKind); 7] = [
    (
        //
        0b000_000..=0b100_111,
        "general",
        AddrKind::General,
    ),
    (
        //
        0b101_000..=0b101_111,
        "parallel read",
        AddrKind::ParallelRead,
    ),
    (
        //
        0b110_000..=0b110_111,
        "external input",
        AddrKind::ExternalInput,
    ),
    (
        //
        0b111_000..=0b111_000,
        "qrr",
        AddrKind::QRR,
    ),
    (
        //
        0b111_001..=0b111_001,
        "rr",
        AddrKind::RR,
    ),
    (
        //
        0b111_010..=0b111_011,
        "high input",
        AddrKind::HighInput,
    ),
    (
        //
        0b111_100..=0b111_111,
        "low input / high z",
        AddrKind::LowInput,
    ),
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddrKind {
    General,
    ParallelRead,
    ExternalInput,
    QRR,
    RR,
    HighInput,
    LowInput,
}

impl AddrKind {
    fn name(self) -> &'static str {
        ADDR_TABLE[self as usize].1
    }
}

impl Display for AddrKind {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        let addr_name =
            ADDR_TABLE.iter().find_map(|(_, addr_name, addr_range)| {
                if addr_range == self {
                    return Some(addr_name);
                }

                None
            });

        write!(
            fmt,
            "{}",
            if let Some(addr_name) = addr_name {
                addr_name
            } else {
                ADDR_TABLE[0].1
            }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Addr(AddrKind, u32);

impl Addr {
    pub fn kind(&self) -> AddrKind {
        self.0
    }

    pub fn name(&self) -> &'static str {
        self.kind().name()
    }

    pub fn val(&self) -> u32 {
        self.1
    }
}

impl Binary for Addr {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{:06b}", self.val())
    }
}

impl Octal for Addr {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{:02o}", self.val())
    }
}

impl From<i32> for Addr {
    fn from(word: i32) -> Addr {
        Addr::from(word as u32)
    }
}

impl From<u8> for Addr {
    fn from(word: u8) -> Addr {
        Addr::from(word as u32)
    }
}

impl From<u32> for Addr {
    fn from(word: u32) -> Addr {
        let addr_bits = (word & ADDR_MASK) >> ADDR_POS;
        let addr_type =
            ADDR_TABLE.iter().find_map(|(addr_range, _, addr_type)| {
                if addr_range.contains(&addr_bits) {
                    return Some(addr_type);
                }

                None
            });

        Addr(
            if let Some(addr_type) = addr_type {
                *addr_type
            } else {
                AddrKind::General
            },
            addr_bits,
        )
    }
}

const CTRL_MASK: u32 = 0b0000_000000_11;
const CTRL_TABLE: [(u32, &str, CtrlKind); 4] = [
    (0b00, "null", CtrlKind::Null),
    (0b01, "copy and shift out", CtrlKind::CopyShift),
    (0b10, "undefined", CtrlKind::Undefined),
    (0b11, "stop tape", CtrlKind::StopTape),
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CtrlKind {
    Null,
    CopyShift,
    Undefined,
    StopTape,
}

impl CtrlKind {
    pub fn name(self) -> &'static str {
        CTRL_TABLE[self as usize].1
    }

    pub fn val(self) -> u32 {
        CTRL_TABLE[self as usize].0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ctrl(CtrlKind);

impl Ctrl {
    pub fn kind(&self) -> CtrlKind {
        self.0
    }

    pub fn name(&self) -> &'static str {
        self.kind().name()
    }

    pub fn val(&self) -> u32 {
        self.kind().val()
    }
}

impl Binary for Ctrl {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{:02b}", self.val())
    }
}

impl Octal for Ctrl {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        write!(fmt, "{:01o}", self.val())
    }
}

impl From<CtrlKind> for Ctrl {
    fn from(kind: CtrlKind) -> Ctrl {
        Ctrl(kind)
    }
}

impl From<i32> for Ctrl {
    fn from(word: i32) -> Ctrl {
        Ctrl::from(word as u32)
    }
}

impl From<u8> for Ctrl {
    fn from(word: u8) -> Ctrl {
        Ctrl::from(word as u32)
    }
}

impl From<u32> for Ctrl {
    fn from(word: u32) -> Ctrl {
        let val = word & CTRL_MASK;

        for (tbl_val, _, tbl_kind) in CTRL_TABLE {
            if tbl_val == val {
                return Ctrl(tbl_kind);
            }
        }

        unreachable!()
    }
}
