use super::{
    DEBUG_PRINT, Span,
    ast::{Iter, Line, Loop, Loops, Range, VarNum},
    compare::compare,
    general::{lines, var_name, var_or_num},
    utils::{close_brace, close_bracket, close_paren, comma, open_brace, open_bracket, open_paren},
};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::{complete::tag, streaming::take_till1},
    character::streaming::multispace0,
    combinator::{into, opt},
    multi::separated_list1,
    sequence::{delimited, preceded},
};
use std::collections::VecDeque;

pub fn for_loop(input: Span) -> IResult<Span, Loop> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a for loop:{}", input);
    }
    let (input, (loop_tag, (inner_name, loop_val), open)) = preceded(
        multispace0,
        (
            tag("for"),
            delimited(open_paren, for_inner, close_paren),
            open_brace,
        ),
    )
    .parse(input)?;
    let (input, (body, close)) = loop_body(input)?;
    let l = Loop::new_with_body(
        Loops::For(loop_tag, inner_name, loop_val),
        body,
        open,
        close,
    );
    Ok((input, l))
}

pub fn list(input: Span) -> IResult<Span, VecDeque<VarNum>> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a list:{}", input);
    }
    let (input, list) = delimited(
        open_bracket,
        separated_list1(comma, var_or_num),
        close_bracket,
    )
    .parse(input)?;
    Ok((input, list.into()))
}

pub fn range(input: Span) -> IResult<Span, Range> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a range:{}", input);
    }
    let (input, _) = opt(tag("(")).parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, start) = var_or_num(input)?;
    let (input, _) = tag("..").parse(input)?;
    let (input, end) = var_or_num(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = opt(tag(")")).parse(input)?;
    Ok((input, Range::new(start, end)))
}

fn for_inner(input: Span) -> IResult<Span, (Span, Iter)> {
    if *DEBUG_PRINT {
        eprintln!(
            "Parsing input for the inner section of the for loop:{}",
            input
        );
    }
    let (input, name) = var_name(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("in").parse(input)?;
    let (input, _) = multispace0(input)?;
    let (input, i) = alt((into(list), into(range), into(var_name))).parse(input)?;
    Ok((input, (name, i)))
}

pub fn while_loop(input: Span) -> IResult<Span, Loop> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a while loop:{}", input);
    }
    let (input, (loop_tag, op, open)) = preceded(
        multispace0,
        (
            tag("while"),
            delimited(open_paren, compare, close_paren),
            open_brace,
        ),
    )
    .parse(input)?;
    let (input, (body, close)) = loop_body(input)?;
    let l = Loop::new_with_body(Loops::While(loop_tag, op), body, open, close);
    Ok((input, l))
}

pub fn if_stmt(input: Span) -> IResult<Span, Loop> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for an if statement:{}", input);
    }
    let (input, (loop_tag, op, open)) = preceded(
        multispace0,
        (
            tag("if"),
            delimited(open_paren, compare, close_paren),
            open_brace,
        ),
    )
    .parse(input)?;
    let (input, (body, close)) = loop_body(input)?;
    let l = Loop::new_with_body(Loops::If(loop_tag, op), body, open, close);
    Ok((input, l))
}

pub fn loops(input: Span) -> IResult<Span, Loop> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a loop:{}", input);
    }
    delimited(
        multispace0,
        alt((if_stmt, while_loop, for_loop)),
        multispace0,
    )
    .parse(input)
}

pub fn loop_body<'a>(input: Span<'a>) -> IResult<Span<'a>, (VecDeque<Line<'a>>, Span<'a>)> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for the lines for the loop body:{}", input);
    }
    let (input, loop_input) = take_till1(|c| c == '}').parse(input)?;
    let (input, brace) = close_brace(input)?;
    let (_, i) = lines(loop_input)?;
    Ok((input, (i.into(), brace)))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::bitops::BitOps;
    use crate::parsers::ast::{BitExpr, Compare, CompareOp, Line, Number, VarNum};

    #[test]
    fn test_for_loop() {
        unsafe {
            let mut l = Loop::new(
                Loops::For(
                    Span::new_from_raw_offset(1, 2, "for", ()),
                    Span::new_from_raw_offset(5, 2, "something", ()),
                    Iter::Var(Span::new_from_raw_offset(18, 2, "somethingElse", ())),
                ),
                Span::new_from_raw_offset(33, 2, "{", ()),
                Span::new_from_raw_offset(55, 4, "}", ()),
            );
            l.add_line(Line::Empty);
            l.add_line(Line::Expr(BitExpr::new(
                VarNum::Var(Span::new_from_raw_offset(39, 3, "i", ())),
                BitOps::RightShift,
                Span::new_from_raw_offset(41, 3, ">>", ()),
                Some(VarNum::Var(Span::new_from_raw_offset(
                    44,
                    3,
                    "something",
                    (),
                ))),
            )));
            l.add_line(Line::Empty);
            l.add_line(Line::Empty);
            assert_eq!(
                for_loop(Span::new(
                    r#"
for(something in somethingElse) {
    i >> something;
}
"#
                )),
                Ok((Span::new_from_raw_offset(57, 5, "", ()), l))
            )
        }
    }

    #[test]
    fn test_while_loop() {
        unsafe {
            let mut l = Loop::new(
                Loops::While(
                    Span::new_from_raw_offset(1, 2, "while", ()),
                    CompareOp::new(
                        VarNum::Var(Span::new_from_raw_offset(7, 2, "something", ())),
                        Compare::Equal,
                        Span::new_from_raw_offset(17, 2, "==", ()),
                        VarNum::Num(Number::new(
                            0,
                            Span::new_from_raw_offset(20, 2, "0", ()),
                            None,
                        )),
                    ),
                ),
                Span::new_from_raw_offset(23, 2, "{", ()),
                Span::new_from_raw_offset(47, 4, "}", ()),
            );
            l.add_line(Line::Empty);
            l.add_line(Line::Expr(BitExpr::new(
                VarNum::Var(Span::new_from_raw_offset(29, 3, "something", ())),
                BitOps::Xor,
                Span::new_from_raw_offset(39, 3, "^", ()),
                Some(VarNum::Num(Number::new(
                    118,
                    Span::new_from_raw_offset(43, 3, "76", ()),
                    Some(Span::new_from_raw_offset(41, 3, "0x", ())),
                ))),
            )));
            l.add_line(Line::Empty);
            l.add_line(Line::Empty);
            assert_eq!(
                while_loop(Span::new(
                    r#"
while(something == 0) {
    something ^ 0x76;
}
"#
                )),
                Ok((Span::new_from_raw_offset(49, 5, "", ()), l))
            )
        }
    }

    #[test]
    fn test_if_statement() {
        unsafe {
            let mut l = Loop::new(
                Loops::If(
                    Span::new_from_raw_offset(1, 2, "if", ()),
                    CompareOp::new(
                        VarNum::Var(Span::new_from_raw_offset(4, 2, "something", ())),
                        Compare::Equal,
                        Span::new_from_raw_offset(14, 2, "==", ()),
                        VarNum::Num(Number::new(
                            0,
                            Span::new_from_raw_offset(17, 2, "0", ()),
                            None,
                        )),
                    ),
                ),
                Span::new_from_raw_offset(20, 2, "{", ()),
                Span::new_from_raw_offset(34, 4, "}", ()),
            );
            l.add_line(Line::Empty);
            l.add_line(Line::Expr(BitExpr::new(
                VarNum::Var(Span::new_from_raw_offset(26, 3, "i", ())),
                BitOps::LeftShift,
                Span::new_from_raw_offset(28, 3, "<<", ()),
                Some(VarNum::Num(Number::new(
                    4,
                    Span::new_from_raw_offset(31, 3, "4", ()),
                    None,
                ))),
            )));
            l.add_line(Line::Empty);
            l.add_line(Line::Empty);
            assert_eq!(
                if_stmt(Span::new(
                    r#"
if(something == 0) {
    i << 4;
}
"#
                )),
                Ok((Span::new_from_raw_offset(36, 5, "", ()), l))
            )
        }
    }
}
