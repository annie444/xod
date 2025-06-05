use super::{
    Span,
    ast::{BoolFunc, Funcs, Range},
    compare::compare,
    general::var_or_num,
    utils::{close_paren, comma, open_paren},
};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::multispace0,
    combinator::into,
    sequence::{pair, separated_pair, terminated},
};

pub fn log_tag(input: Span) -> IResult<Span, Span> {
    let (input, func) = tag("log").parse_complete(input)?;
    Ok((input, func))
}

pub fn log_func(input: Span) -> IResult<Span, Funcs> {
    let (input, (func, (right, left))) = (
        terminated(log_tag, open_paren),
        terminated(separated_pair(var_or_num, comma, var_or_num), close_paren),
    )
        .parse_complete(input)?;
    Ok((input, Funcs::Log(func, right, left)))
}

pub fn bool_tag(input: Span) -> IResult<Span, Span> {
    let (input, func) = tag("bool").parse_complete(input)?;
    Ok((input, func))
}

pub fn bool_func(input: Span) -> IResult<Span, (Span, BoolFunc)> {
    let (input, (func, body)) = (
        terminated(bool_tag, open_paren),
        terminated(alt((into(compare), into(var_or_num))), close_paren),
    )
        .parse_complete(input)?;
    Ok((input, (func, body)))
}

fn history_tag(input: Span) -> IResult<Span, Span> {
    let (input, func) = alt((tag("history"), tag("hist"))).parse_complete(input)?;
    Ok((input, func))
}

pub fn history_func(input: Span) -> IResult<Span, Funcs> {
    let (input, func) =
        terminated(history_tag, pair(open_paren, close_paren)).parse_complete(input)?;
    Ok((input, Funcs::History(func)))
}

fn clear_tag(input: Span) -> IResult<Span, Span> {
    let (input, func) = tag("clear").parse_complete(input)?;
    Ok((input, func))
}

pub fn clear_func(input: Span) -> IResult<Span, Funcs> {
    let (input, func) =
        terminated(clear_tag, pair(open_paren, close_paren)).parse_complete(input)?;
    Ok((input, Funcs::Clear(func)))
}

fn quit_tag(input: Span) -> IResult<Span, Span> {
    let (input, func) = alt((tag("quit"), tag("exit"))).parse_complete(input)?;
    Ok((input, func))
}

pub fn quit_func(input: Span) -> IResult<Span, Funcs> {
    let (input, func) =
        terminated(quit_tag, pair(open_paren, close_paren)).parse_complete(input)?;
    Ok((input, Funcs::Quit(func)))
}

fn help_tag(input: Span) -> IResult<Span, Span> {
    let (input, func) = tag("help").parse_complete(input)?;
    Ok((input, func))
}

pub fn help_func(input: Span) -> IResult<Span, Funcs> {
    let (input, func) =
        terminated(help_tag, pair(open_paren, close_paren)).parse_complete(input)?;
    Ok((input, Funcs::Help(func)))
}

pub fn hex_tag(input: Span) -> IResult<Span, Span> {
    let (input, func) = tag("hex").parse_complete(input)?;
    Ok((input, func))
}

pub fn hex_func(input: Span) -> IResult<Span, Funcs> {
    let (input, (func, body)) = (
        terminated(hex_tag, open_paren),
        terminated(var_or_num, close_paren),
    )
        .parse_complete(input)?;
    Ok((input, Funcs::Hex(func, body)))
}

pub fn bin_tag(input: Span) -> IResult<Span, Span> {
    let (input, func) = tag("bin").parse_complete(input)?;
    Ok((input, func))
}

pub fn bin_func(input: Span) -> IResult<Span, Funcs> {
    let (input, (func, body)) = (
        terminated(bin_tag, open_paren),
        terminated(var_or_num, close_paren),
    )
        .parse_complete(input)?;
    Ok((input, Funcs::Bin(func, body)))
}

pub fn oct_tag(input: Span) -> IResult<Span, Span> {
    let (input, func) = tag("oct").parse_complete(input)?;
    Ok((input, func))
}

pub fn oct_func(input: Span) -> IResult<Span, Funcs> {
    let (input, (func, body)) = (
        terminated(oct_tag, open_paren),
        terminated(var_or_num, close_paren),
    )
        .parse_complete(input)?;
    Ok((input, Funcs::Oct(func, body)))
}

pub fn dec_tag(input: Span) -> IResult<Span, Span> {
    let (input, func) = tag("dec").parse_complete(input)?;
    Ok((input, func))
}

pub fn dec_func(input: Span) -> IResult<Span, Funcs> {
    let (input, (func, body)) = (
        terminated(dec_tag, open_paren),
        terminated(var_or_num, close_paren),
    )
        .parse_complete(input)?;
    Ok((input, Funcs::Dec(func, body)))
}

pub fn funcs(input: Span) -> IResult<Span, Funcs> {
    let (input, _) = multispace0(input)?;
    alt((
        into(bool_func),
        quit_func,
        history_func,
        clear_func,
        log_func,
        hex_func,
        oct_func,
        bin_func,
        dec_func,
        help_func,
    ))
    .parse_complete(input)
}

pub fn range_func(input: Span) -> IResult<Span, Range> {
    let (input, (func, (start, end))) = (
        terminated(tag("range"), open_paren),
        terminated(separated_pair(var_or_num, comma, var_or_num), close_paren),
    )
        .parse_complete(input)?;
    Ok((input, Range::new(func, start, end)))
}

#[cfg(test)]
mod test {
    use crate::bitops::BitOps;
    use crate::parsers::ast::{
        BitExpr, BoolFunc, Compare, CompareOp, Funcs, Line, Number, SepBitExpr, VarNum,
    };
    use crate::parsers::general::lines;
    use std::collections::VecDeque;

    use super::*;

    #[test]
    fn test_log_function() {
        let line = r#"
log(0x10, 2)
"#;
        let span = Span::new(line);
        let result = lines(span);
        unsafe {
            assert_eq!(
                result,
                Ok((
                    Span::new_from_raw_offset(14, 3, "", ()),
                    VecDeque::from([Line::Func(Funcs::Log(
                        Span::new_from_raw_offset(1, 2, "log", ()),
                        VarNum::Num(Number::new(
                            16,
                            Span::new_from_raw_offset(7, 2, "10", ()),
                            Some(Span::new_from_raw_offset(5, 2, "0x", ()))
                        )),
                        VarNum::Num(Number::new(
                            2,
                            Span::new_from_raw_offset(11, 2, "2", ()),
                            None
                        )),
                    ))])
                ))
            )
        }
    }
    #[test]
    fn test_range() {
        let line = r#"
for(i in range(0, 6)) {
    j = 1 << i
    bin(i)
    bin(j)
}
"#;
        let span = Span::new(line);
        let result = lines(span);
        assert!(
            result.is_ok(),
            "Failed to parse range function: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_exit() {
        unsafe {
            assert_eq!(
                quit_func(Span::new("exit()")),
                Ok((
                    Span::new_from_raw_offset(6, 1, "", ()),
                    Funcs::Quit(Span::new_from_raw_offset(0, 1, "exit", ()))
                ))
            );
            assert_eq!(
                funcs(Span::new("exit()")),
                Ok((
                    Span::new_from_raw_offset(6, 1, "", ()),
                    Funcs::Quit(Span::new_from_raw_offset(0, 1, "exit", ()))
                ))
            )
        }
    }

    #[test]
    fn test_quit() {
        unsafe {
            assert_eq!(
                quit_func(Span::new("quit()")),
                Ok((
                    Span::new_from_raw_offset(6, 1, "", ()),
                    Funcs::Quit(Span::new_from_raw_offset(0, 1, "quit", ()))
                ))
            );
            assert_eq!(
                funcs(Span::new("quit()")),
                Ok((
                    Span::new_from_raw_offset(6, 1, "", ()),
                    Funcs::Quit(Span::new_from_raw_offset(0, 1, "quit", ()))
                ))
            )
        }
    }

    #[test]
    fn test_simple_number_bool_function() {
        unsafe {
            assert_eq!(
                bool_func(Span::new("bool(0)")),
                Ok((
                    Span::new_from_raw_offset(7, 1, "", ()),
                    (
                        Span::new_from_raw_offset(0, 1, "bool", ()),
                        BoolFunc::VarNum(VarNum::Num(Number::new(
                            0,
                            Span::new_from_raw_offset(5, 1, "0", ()),
                            None
                        )))
                    )
                ))
            );
            assert_eq!(
                funcs(Span::new("bool(0)")),
                Ok((
                    Span::new_from_raw_offset(7, 1, "", ()),
                    Funcs::Bool(
                        Span::new_from_raw_offset(0, 1, "bool", ()),
                        BoolFunc::VarNum(VarNum::Num(Number::new(
                            0,
                            Span::new_from_raw_offset(5, 1, "0", ()),
                            None
                        )))
                    )
                ))
            )
        }
    }

    #[test]
    fn test_simple_variable_bool_function() {
        unsafe {
            assert_eq!(
                bool_func(Span::new("bool(someVar)")),
                Ok((
                    Span::new_from_raw_offset(13, 1, "", ()),
                    (
                        Span::new_from_raw_offset(0, 1, "bool", ()),
                        BoolFunc::VarNum(VarNum::Var(Span::new_from_raw_offset(
                            5,
                            1,
                            "someVar",
                            ()
                        )))
                    )
                ))
            );
            assert_eq!(
                funcs(Span::new("bool(someVar)")),
                Ok((
                    Span::new_from_raw_offset(13, 1, "", ()),
                    Funcs::Bool(
                        Span::new_from_raw_offset(0, 1, "bool", ()),
                        BoolFunc::VarNum(VarNum::Var(Span::new_from_raw_offset(
                            5,
                            1,
                            "someVar",
                            ()
                        )))
                    )
                ))
            )
        }
    }

    #[test]
    fn test_simple_comparison_bool_function() {
        unsafe {
            assert_eq!(
                bool_func(Span::new("bool(0 < 1)")),
                Ok((
                    Span::new_from_raw_offset(11, 1, "", ()),
                    (
                        Span::new_from_raw_offset(0, 1, "bool", ()),
                        BoolFunc::Compare(CompareOp::new(
                            VarNum::Num(Number::new(
                                0,
                                Span::new_from_raw_offset(5, 1, "0", ()),
                                None
                            )),
                            Compare::Less,
                            Span::new_from_raw_offset(7, 1, "<", ()),
                            VarNum::Num(Number::new(
                                1,
                                Span::new_from_raw_offset(9, 1, "1", ()),
                                None
                            )),
                        ))
                    )
                ))
            );
            assert_eq!(
                funcs(Span::new("bool(0 < 1)")),
                Ok((
                    Span::new_from_raw_offset(11, 1, "", ()),
                    Funcs::Bool(
                        Span::new_from_raw_offset(0, 1, "bool", ()),
                        BoolFunc::Compare(CompareOp::new(
                            VarNum::Num(Number::new(
                                0,
                                Span::new_from_raw_offset(5, 1, "0", ()),
                                None
                            )),
                            Compare::Less,
                            Span::new_from_raw_offset(7, 1, "<", ()),
                            VarNum::Num(Number::new(
                                1,
                                Span::new_from_raw_offset(9, 1, "1", ()),
                                None
                            )),
                        ))
                    )
                ))
            );
        }
    }

    #[test]
    fn test_complex_expression_bool_function() {
        unsafe {
            assert_eq!(
                bool_func(Span::new("bool((0x16 << 2) > 1)")),
                Ok((
                    Span::new_from_raw_offset(21, 1, "", ()),
                    (
                        Span::new_from_raw_offset(0, 1, "bool", ()),
                        BoolFunc::Compare(CompareOp::new(
                            VarNum::Expr(Box::new(SepBitExpr::new(
                                Span::new_from_raw_offset(5, 1, "(", ()),
                                BitExpr::new(
                                    VarNum::Num(Number::new(
                                        22,
                                        Span::new_from_raw_offset(8, 1, "16", ()),
                                        Some(Span::new_from_raw_offset(6, 1, "0x", ()))
                                    )),
                                    BitOps::LeftShift,
                                    Span::new_from_raw_offset(11, 1, "<<", ()),
                                    Some(VarNum::Num(Number::new(
                                        2,
                                        Span::new_from_raw_offset(14, 1, "2", ()),
                                        None
                                    )))
                                ),
                                Span::new_from_raw_offset(15, 1, ")", ())
                            ))),
                            Compare::Greater,
                            Span::new_from_raw_offset(17, 1, ">", ()),
                            VarNum::Num(Number::new(
                                1,
                                Span::new_from_raw_offset(19, 1, "1", ()),
                                None
                            )),
                        ))
                    )
                ))
            );
            assert_eq!(
                funcs(Span::new("bool((0x16 << 2) > 1)")),
                Ok((
                    Span::new_from_raw_offset(21, 1, "", ()),
                    Funcs::Bool(
                        Span::new_from_raw_offset(0, 1, "bool", ()),
                        BoolFunc::Compare(CompareOp::new(
                            VarNum::Expr(Box::new(SepBitExpr::new(
                                Span::new_from_raw_offset(5, 1, "(", ()),
                                BitExpr::new(
                                    VarNum::Num(Number::new(
                                        22,
                                        Span::new_from_raw_offset(8, 1, "16", ()),
                                        Some(Span::new_from_raw_offset(6, 1, "0x", ()))
                                    )),
                                    BitOps::LeftShift,
                                    Span::new_from_raw_offset(11, 1, "<<", ()),
                                    Some(VarNum::Num(Number::new(
                                        2,
                                        Span::new_from_raw_offset(14, 1, "2", ()),
                                        None
                                    )))
                                ),
                                Span::new_from_raw_offset(15, 1, ")", ())
                            ))),
                            Compare::Greater,
                            Span::new_from_raw_offset(17, 1, ">", ()),
                            VarNum::Num(Number::new(
                                1,
                                Span::new_from_raw_offset(19, 1, "1", ()),
                                None
                            )),
                        ))
                    )
                ))
            )
        }
    }

    #[test]
    fn test_bool_function_in_loop() {
        let line = r#"
if(bool(other) == 1) {
    hex(true)
}
"#;
        let span = Span::new(line);
        let result = lines(span);
        assert!(
            result.is_ok(),
            "Failed to parse bool function in loop: {:?}",
            result.err()
        );
    }
}
