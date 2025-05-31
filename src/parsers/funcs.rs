use super::{DEBUG_PRINT, Span, ast::Funcs, compare::compare, general::var_or_num};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::multispace0,
    combinator::into,
    sequence::{delimited, pair, terminated},
};

pub fn bool_func(input: Span) -> IResult<Span, Funcs> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a bool function:{}", input);
    }
    let (input, func) = tag("bool").parse(input)?;
    let (input, body) = delimited(
        pair(tag("("), multispace0),
        alt((into(compare), into(var_or_num))),
        pair(multispace0, tag(")")),
    )
    .parse(input)?;
    Ok((input, Funcs::Bool(func, body)))
}

pub fn quit_func(input: Span) -> IResult<Span, Funcs> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a quit function:{}", input);
    }
    let (input, func) = terminated(
        alt((tag("quit"), tag("exit"))),
        pair(pair(tag("("), multispace0), pair(multispace0, tag(")"))),
    )
    .parse(input)?;
    Ok((input, Funcs::Quit(func)))
}

pub fn run_func(input: Span) -> IResult<Span, Funcs> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a run function:{}", input);
    }
    let (input, func) = terminated(
        alt((tag("run"), tag("eval"), tag("exec"))),
        pair(pair(tag("("), multispace0), pair(multispace0, tag(")"))),
    )
    .parse(input)?;
    Ok((input, Funcs::Run(func)))
}

pub fn funcs(input: Span) -> IResult<Span, Funcs> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a function:{}", input);
    }
    let (input, _) = multispace0(input)?;
    alt((bool_func, quit_func, run_func)).parse(input)
}

#[cfg(test)]
mod test {
    use crate::bitops::BitOps;
    use crate::parsers::ast::{
        BitExpr, BoolFunc, Compare, CompareOp, Funcs, Number, SepBitExpr, VarNum,
    };

    use super::*;

    #[test]
    fn test_run() {
        unsafe {
            assert_eq!(
                run_func(Span::new("run()")),
                Ok((
                    Span::new_from_raw_offset(5, 1, "", ()),
                    Funcs::Run(Span::new_from_raw_offset(0, 1, "run", ()))
                ))
            );
            assert_eq!(
                funcs(Span::new("run()")),
                Ok((
                    Span::new_from_raw_offset(5, 1, "", ()),
                    Funcs::Run(Span::new_from_raw_offset(0, 1, "run", ()))
                ))
            );
        }
    }

    #[test]
    fn test_eval() {
        unsafe {
            assert_eq!(
                run_func(Span::new("eval()")),
                Ok((
                    Span::new_from_raw_offset(6, 1, "", ()),
                    Funcs::Run(Span::new_from_raw_offset(0, 1, "eval", ()))
                ))
            );
            assert_eq!(
                funcs(Span::new("eval()")),
                Ok((
                    Span::new_from_raw_offset(6, 1, "", ()),
                    Funcs::Run(Span::new_from_raw_offset(0, 1, "eval", ()))
                ))
            );
        }
    }

    #[test]
    fn test_exec() {
        unsafe {
            assert_eq!(
                run_func(Span::new("exec()")),
                Ok((
                    Span::new_from_raw_offset(6, 1, "", ()),
                    Funcs::Run(Span::new_from_raw_offset(0, 1, "exec", ()))
                ))
            );
            assert_eq!(
                funcs(Span::new("exec()")),
                Ok((
                    Span::new_from_raw_offset(6, 1, "", ()),
                    Funcs::Run(Span::new_from_raw_offset(0, 1, "exec", ()))
                ))
            );
        }
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
                    Funcs::Bool(
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
                    Funcs::Bool(
                        Span::new_from_raw_offset(0, 1, "bool", ()),
                        BoolFunc::Compare(CompareOp::new(
                            VarNum::Expr(Box::new(SepBitExpr::new(
                                Span::new_from_raw_offset(7, 1, "(", ()),
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
}
