use crate::data::*;
use chonk::prelude::*;
use SyntaxError::*;

#[derive(Debug)]
pub enum SyntaxError {
    ExpectedInst(Inst),
    ExpectedAddr,
    ExpectedCtrl,
    ExpectedWord,
    ExpectedComment,
    UnexpectedEoi,
}

#[allow(clippy::redundant_closure)]
pub fn nodes<'a>() -> impl Parser<'a, Nodes, SyntaxError> {
    move |ctx| {
        trim(find_until(eoi(), trim(find_any((comment(), word())))))
            .parse(ctx)
            .map_result(|nodes| Nodes(nodes))
    }
}

#[test]
fn parse_nodes() {
    let asm = r"
    ONE 0o77 0b0
    STOC 0o50 0b0
    STOC 0o51 0b0
    STO 0o52 0b0
    STOC 0o53 0b0
    STO 0o54 0b0
    NOP0 0o77 0b1
    ";

    let expected = Nodes(vec![
        Node::Word(
            Inst::from(InstKind::One),
            Addr::from(63 << ADDR_POS),
            Ctrl::from(CtrlKind::Null),
        ),
        Node::Word(
            Inst::from(InstKind::StoC),
            Addr::from(40 << ADDR_POS),
            Ctrl::from(CtrlKind::Null),
        ),
        Node::Word(
            Inst::from(InstKind::StoC),
            Addr::from(41 << ADDR_POS),
            Ctrl::from(CtrlKind::Null),
        ),
        Node::Word(
            Inst::from(InstKind::Sto),
            Addr::from(42 << ADDR_POS),
            Ctrl::from(CtrlKind::Null),
        ),
        Node::Word(
            Inst::from(InstKind::StoC),
            Addr::from(43 << ADDR_POS),
            Ctrl::from(CtrlKind::Null),
        ),
        Node::Word(
            Inst::from(InstKind::Sto),
            Addr::from(44 << ADDR_POS),
            Ctrl::from(CtrlKind::Null),
        ),
        Node::Word(
            Inst::from(InstKind::Nop0),
            Addr::from(63 << ADDR_POS),
            Ctrl::from(CtrlKind::CopyShift),
        ),
    ]);

    assert_eq!(expected, nodes().parse(asm).unwrap_result());
}

fn bin<'a>() -> impl Parser<'a, u32, SyntaxError> {
    move |ctx| {
        find_all((is("0b"), take(1..32, is(one_of("01")))))
            .parse(ctx)
            .map_result(|(_, bin)| u32::from_str_radix(bin, 2).unwrap())
            .map_error(|err| err.with_message(ExpectedAddr))
    }
}

fn oct<'a>() -> impl Parser<'a, u32, SyntaxError> {
    move |ctx| {
        find_all((is("0o"), take(1..11, is(one_of("01234567")))))
            .parse(ctx)
            .map_result(|(_, octal)| u32::from_str_radix(octal, 8).unwrap())
            .map_error(|err| err.with_message(ExpectedAddr))
    }
}

fn hex<'a>() -> impl Parser<'a, u32, SyntaxError> {
    move |ctx| {
        find_all((is("0h"), take(1..8, is(hex_digit))))
            .parse(ctx)
            .map_result(|(_, hex)| u32::from_str_radix(hex, 16).unwrap())
            .map_error(|err| err.with_message(ExpectedAddr))
    }
}

fn newline<'a>() -> impl Parser<'a, &'a str, SyntaxError> {
    move |ctx| take_any((eoi(), take_any((is("\n"), is("\r\n"))))).parse(ctx)
}

fn comment<'a>() -> impl Parser<'a, Node, SyntaxError> {
    move |ctx| {
        find_all((is(';'), take_until(newline(), is(any)), newline()))
            .parse(ctx)
            .map_result(|(_, text, ..)| Node::Comment(text.trim_end().into()))
            .map_error(|err| err.with_message(ExpectedComment))
    }
}

#[test]
fn parse_comment() {
    assert_eq!(
        Node::Comment("ONE 0o77 00".into()),
        comment().parse(";ONE 0o77 00").unwrap_result()
    );
    assert_eq!(
        Node::Comment("".into()),
        comment().parse(";   \n").unwrap_result()
    );
    assert_eq!(
        Node::Comment(" foo bar".into()),
        comment().parse("; foo bar  \n").unwrap_result()
    );
}

fn word<'a>() -> impl Parser<'a, Node, SyntaxError> {
    move |ctx| {
        find_all((inst(), space(1..), addr(), space(1..), ctrl(), newline()))
            .parse(ctx)
            .map_result(|(inst, _, addr, _, ctrl, ..)| {
                Node::Word(inst, addr, ctrl)
            })
    }
}

#[test]
fn parse_word() {
    assert_eq!(
        Node::Word(
            Inst::from(InstKind::One),
            Addr::from(63 << ADDR_POS),
            Ctrl::from(CtrlKind::Null)
        ),
        word().parse("ONE 0o77 0h0").unwrap_result()
    );
}

fn inst<'a>() -> impl Parser<'a, Inst, SyntaxError> {
    move |ctx| {
        find_any((
            inst_item(Inst::from(InstKind::Nop0), "Nop0"),
            inst_item(Inst::from(InstKind::Ld), "Ld"),
            inst_item(Inst::from(InstKind::Add), "Add"),
            inst_item(Inst::from(InstKind::Sub), "Sub"),
            inst_item(Inst::from(InstKind::One), "One"),
            inst_item(Inst::from(InstKind::Nand), "Nand"),
            inst_item(Inst::from(InstKind::Or), "Or"),
            inst_item(Inst::from(InstKind::Xor), "Xor"),
            inst_item(Inst::from(InstKind::StoC), "StoC"),
            inst_item(Inst::from(InstKind::Sto), "Sto"),
            inst_item(Inst::from(InstKind::Ien), "Ien"),
            inst_item(Inst::from(InstKind::Oen), "Oen"),
            inst_item(Inst::from(InstKind::Ioc), "Ioc"),
            inst_item(Inst::from(InstKind::Rtn), "Rtn"),
            inst_item(Inst::from(InstKind::Skz), "Skz"),
            inst_item(Inst::from(InstKind::NopF), "NopF"),
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
    assert_eq!(
        Inst::from(InstKind::Nop0),
        inst().parse("Nop0").unwrap_result()
    );
    assert_eq!(
        Inst::from(InstKind::Nop0),
        inst().parse("nop0").unwrap_result()
    );
    assert_eq!(
        Inst::from(InstKind::Nop0),
        inst().parse("NOP0").unwrap_result()
    );
    assert_eq!(
        Inst::from(InstKind::StoC),
        inst().parse("stoc").unwrap_result()
    );
}

fn addr<'a>() -> impl Parser<'a, Addr, SyntaxError> {
    move |ctx| {
        find_any((bin(), oct(), hex()))
            .parse(ctx)
            .map_result(|addr| Addr::from(addr << ADDR_POS))
            .map_error(|err| err.with_message(ExpectedAddr))
    }
}

#[test]
fn parse_addr() {
    assert_eq!(
        Addr::from(63 << ADDR_POS),
        addr().parse("0o77").unwrap_result()
    );
    assert_eq!(
        Addr::from(0 << ADDR_POS),
        addr().parse("0o0").unwrap_result()
    );
    assert!(addr().parse("0o88").is_err());
    assert!(addr().parse("123").is_err());
    assert!(addr().parse("").is_err());
}

fn ctrl<'a>() -> impl Parser<'a, Ctrl, SyntaxError> {
    move |ctx| {
        find_any((bin(), oct(), hex()))
            .parse(ctx)
            .map_result(Ctrl::from)
            .map_error(|err| err.with_message(ExpectedCtrl))
    }
}

#[test]
fn parse_ctrl() {
    assert_eq!(
        Ctrl::from(CtrlKind::Null),
        ctrl().parse("0b00").unwrap_result()
    );
    assert_eq!(
        Ctrl::from(CtrlKind::CopyShift),
        ctrl().parse("0b01").unwrap_result()
    );
    assert_eq!(
        Ctrl::from(CtrlKind::Undefined),
        ctrl().parse("0b10").unwrap_result()
    );
    assert_eq!(
        Ctrl::from(CtrlKind::StopTape),
        ctrl().parse("0b11").unwrap_result()
    );
    assert_eq!(
        Ctrl::from(CtrlKind::Null),
        ctrl().parse("0h0").unwrap_result()
    );
    assert_eq!(
        Ctrl::from(CtrlKind::Null),
        ctrl().parse("0b0").unwrap_result()
    );
    assert!(ctrl().parse("0b22").is_err());
    assert!(ctrl().parse("22").is_err());
    assert!(ctrl().parse("").is_err());
}
