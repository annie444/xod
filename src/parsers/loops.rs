use super::{
    DEBUG_PRINT, Span,
    ast::{Iter, Line, Loop, Loops, VarNum},
    compare::compare,
    funcs::range_func,
    general::{line, var_name, var_or_num},
    utils::{
        close_brace, close_bracket, close_paren, comma, open_brace, open_bracket, open_paren,
        opt_multispace0,
    },
};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_till1},
    character::complete::multispace0,
    combinator::{eof, into},
    multi::{many0, separated_list0},
    sequence::{delimited, preceded, terminated},
};
use std::collections::VecDeque;
use std::sync::Mutex;

pub fn for_loop(input: Span) -> IResult<Span, Loop> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a for loop:{}", input);
    }
    let (input, (loop_tag, (inner_name, loop_val), open)) = preceded(
        multispace0,
        (
            terminated(tag("for"), opt_multispace0),
            delimited(open_paren, for_inner, close_paren),
            open_brace,
        ),
    )
    .parse_complete(input)?;
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
        preceded(opt_multispace0, open_bracket),
        separated_list0(comma, var_or_num),
        close_bracket,
    )
    .parse_complete(input)?;
    Ok((input, list.into()))
}

fn for_inner(input: Span) -> IResult<Span, (Span, Iter)> {
    if *DEBUG_PRINT {
        eprintln!(
            "Parsing input for the inner section of the for loop:{}",
            input
        );
    }
    let (input, name) = terminated(var_name, opt_multispace0).parse_complete(input)?;
    let (input, _) = terminated(tag("in"), opt_multispace0).parse_complete(input)?;
    let (input, i) = alt((into(list), into(range_func), into(var_name))).parse_complete(input)?;
    Ok((input, (name, i)))
}

pub fn while_loop(input: Span) -> IResult<Span, Loop> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a while loop:{}", input);
    }
    let (input, (loop_tag, op, open)) = preceded(
        multispace0,
        (
            terminated(tag("while"), opt_multispace0),
            delimited(open_paren, compare, close_paren),
            open_brace,
        ),
    )
    .parse_complete(input)?;
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
            terminated(tag("if"), opt_multispace0),
            delimited(open_paren, compare, close_paren),
            open_brace,
        ),
    )
    .parse_complete(input)?;
    let (input, (body, close)) = loop_body(input)?;
    let l = Loop::new_with_body(Loops::If(loop_tag, op), body, open, close);
    Ok((input, l))
}

pub fn loops(input: Span) -> IResult<Span, Loop> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a loop:{}", input);
    }
    preceded(multispace0, alt((if_stmt, while_loop, for_loop))).parse_complete(input)
}

fn brace_count() -> impl Fn(char) -> bool {
    let brace_count = Mutex::new(0);
    move |c| {
        if brace_count.is_poisoned() {
            brace_count.clear_poison();
        }
        let mut bc = brace_count.lock().unwrap();
        if c == '{' {
            *bc += 1;
            false
        } else if c == '}' {
            if *bc == 0 {
                true
            } else {
                *bc -= 1;
                false
            }
        } else {
            false
        }
    }
}

pub fn loop_body(input: Span<'_>) -> IResult<Span<'_>, (VecDeque<Line<'_>>, Span<'_>)> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for the lines for the loop body:{}", input);
    }
    let (input, loop_input) = take_till1(brace_count()).parse_complete(input)?;
    let (input, brace) = close_brace(input)?;
    let (_, i) = terminated(into(many0(line)), eof).parse_complete(loop_input)?;
    Ok((input, (i, brace)))
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
                    Span::new_from_raw_offset(6, 2, "something", ()),
                    Iter::Var(Span::new_from_raw_offset(19, 2, "somethingElse", ())),
                ),
                Span::new_from_raw_offset(34, 2, "{", ()),
                Span::new_from_raw_offset(55, 4, "}", ()),
            );
            l.add_line(Line::Expr(BitExpr::new(
                VarNum::Var(Span::new_from_raw_offset(40, 3, "i", ())),
                BitOps::RightShift,
                Span::new_from_raw_offset(42, 3, ">>", ()),
                Some(VarNum::Var(Span::new_from_raw_offset(
                    45,
                    3,
                    "something",
                    (),
                ))),
            )));
            let test = (for_loop).parse_complete(Span::new(
                r#"
for (something in somethingElse) {
    i >> something
}
"#,
            ));
            assert_eq!(test, Ok((Span::new_from_raw_offset(56, 4, "\n", ()), l)))
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
                Span::new_from_raw_offset(46, 4, "}", ()),
            );
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
            let test = (while_loop).parse_complete(Span::new(
                r#"
while(something == 0) {
    something ^ 0x76
}
"#,
            ));
            assert_eq!(test, Ok((Span::new_from_raw_offset(47, 4, "\n", ()), l)))
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
                Span::new_from_raw_offset(33, 4, "}", ()),
            );
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
            let test = (if_stmt).parse_complete(Span::new(
                r#"
if(something == 0) {
    i << 4
}
"#,
            ));
            assert_eq!(test, Ok((Span::new_from_raw_offset(34, 4, "\n", ()), l)))
        }
    }
}
