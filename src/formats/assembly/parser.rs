use crate::data::*;
use chonk::prelude::*;
use SyntaxError::*;

#[derive(Debug)]
pub enum SyntaxError {
    ExpectedInst(Inst),
    ExpectedAddr,
    ExpectedCtrl,
    ExpectedWord,
    UnexpectedEoi,
}

#[allow(clippy::redundant_closure)]
pub fn words<'a>() -> impl Parser<'a, Words, SyntaxError> {
    move |ctx| {
        trim(find_until(
            eoi(),
            trim(word()),
        ))
            .parse(ctx)
            .map_result(|words| Words(words))
    }
}

#[test]
fn parse_words() {
    let asm = r"
ONE 0o77 00
STOC 0o50 00
STOC 0o51 00
STO 0o52 00
STOC 0o53 00
STO 0o54 00
NOP0 0o77 01";

    let expected = Words(vec![
        Word(Inst::One, Addr(63), Ctrl(0)),
        Word(Inst::StoC, Addr(40), Ctrl(0)),
        Word(Inst::StoC, Addr(41), Ctrl(0)),
        Word(Inst::Sto, Addr(42), Ctrl(0)),
        Word(Inst::StoC, Addr(43), Ctrl(0)),
        Word(Inst::Sto, Addr(44), Ctrl(0)),
        Word(Inst::Nop0, Addr(63), Ctrl(1)),
    ]);

    assert_eq!(expected, words().parse(asm).unwrap_result());
}

fn newline<'a>() -> impl Parser<'a, &'a str, SyntaxError> {
    move |ctx| {
        take_any((
            is("\n"),
            is("\r\n"),
        )).parse(ctx)
    }
}

fn word<'a>() -> impl Parser<'a, Word, SyntaxError> {
    move |ctx| {
        find_all((
            inst(),
            space(1..),
            addr(),
            space(1..),
            ctrl(),
            take_any((newline(), eoi())),
        ))
            .parse(ctx)
            .map_result(|(inst, _, addr, _, ctrl, ..)| Word(inst, addr, ctrl))
    }
}

#[test]
fn parse_word() {
    assert_eq!(Word(Inst::One, Addr(63), Ctrl(0)), word().parse("ONE 0o77 00").unwrap_result());
}

fn inst<'a>() -> impl Parser<'a, Inst, SyntaxError> {
    move |ctx| {
        find_any((
            inst_item(Inst::Nop0, "Nop0"),
            inst_item(Inst::Ld, "Ld"),
            inst_item(Inst::Add, "Add"),
            inst_item(Inst::Sub, "Sub"),
            inst_item(Inst::One, "One"),
            inst_item(Inst::Nand, "Nand"),
            inst_item(Inst::Or, "Or"),
            inst_item(Inst::Xor, "Xor"),
            inst_item(Inst::StoC, "StoC"),
            inst_item(Inst::Sto, "Sto"),
            inst_item(Inst::Ien, "Ien"),
            inst_item(Inst::Oen, "Oen"),
            inst_item(Inst::Ioc, "Ioc"),
            inst_item(Inst::Rtn, "Rtn"),
            inst_item(Inst::Skz, "Skz"),
            inst_item(Inst::NopF, "NopF"),
        ))
        .parse(ctx)
    }
}

fn inst_item(inst: Inst, name: &str) -> impl Parser<'_, Inst, SyntaxError> {
    move |ctx| {
        let lower_name = name.to_lowercase();
        let upper_name = name.to_uppercase();

        take_any((is(name), is(lower_name), is(upper_name)))
            .parse(ctx)
            .map_result(|_| inst)
            .map_error(|err| err.with_message(ExpectedInst(inst)))
    }
}

#[test]
fn parse_inst() {
    assert_eq!(Inst::Nop0, inst().parse("Nop0").unwrap_result());
    assert_eq!(Inst::Nop0, inst().parse("nop0").unwrap_result());
    assert_eq!(Inst::Nop0, inst().parse("NOP0").unwrap_result());
    assert_eq!(Inst::StoC, inst().parse("stoc").unwrap_result());
}

fn addr<'a>() -> impl Parser<'a, Addr, SyntaxError> {
    move |ctx| {
        find_all((is("0o"), take(1..2, is(one_of("01234567")))))
            .parse(ctx)
            .map_result(|(_, addr)| Addr::from(u32::from_str_radix(addr, 8).unwrap() << ADDR_POS))
            .map_error(|err| err.with_message(ExpectedAddr))
    }
}

#[test]
fn parse_addr() {
    assert_eq!(Addr(63), addr().parse("0o77123").unwrap_result());
    assert_eq!(Addr(0), addr().parse("0o0").unwrap_result());
    assert!(addr().parse("0o88").is_err());
    assert!(addr().parse("123").is_err());
    assert!(addr().parse("").is_err());
}

fn ctrl<'a>() -> impl Parser<'a, Ctrl, SyntaxError> {
    move |ctx| {
        find_all((optional(is("0b")), take(1..2, is(one_of("01")))))
            .parse(ctx)
            .map_result(|(_, ctrl)| Ctrl(u32::from_str_radix(ctrl, 2).unwrap()))
            .map_error(|err| err.with_message(ExpectedCtrl))
    }
}

#[test]
fn parse_ctrl() {
    assert_eq!(Ctrl(0), ctrl().parse("0b00").unwrap_result());
    assert_eq!(Ctrl(1), ctrl().parse("0b01").unwrap_result());
    assert_eq!(Ctrl(2), ctrl().parse("0b10").unwrap_result());
    assert_eq!(Ctrl(3), ctrl().parse("0b11").unwrap_result());
    assert_eq!(Ctrl(0), ctrl().parse("0").unwrap_result());
    assert_eq!(Ctrl(0), ctrl().parse("0b0").unwrap_result());
    assert!(ctrl().parse("0b22").is_err());
    assert!(ctrl().parse("22").is_err());
    assert!(ctrl().parse("").is_err());
}