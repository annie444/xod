use super::{
    DEBUG_PRINT, Span,
    ast::{Line, VarNum, VarOrVal, Variable},
    bitops::{expr, sep_expr},
    compare::compare,
    funcs::funcs,
    loops::{list, loops, range},
    numbers::num,
    utils::{opt_multispace0, space_around},
};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, char, newline},
    combinator::{eof, into, recognize},
    multi::{many0, many1},
    sequence::{pair, terminated},
};
use std::collections::VecDeque;

pub fn assign(input: Span) -> IResult<Span, char> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a assignment char:{}", input);
    }
    char('=').parse_complete(input)
}

pub fn var_name(input: Span) -> IResult<Span, Span> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a variable name:{}", input);
    }
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_")))),
    ))
    .parse_complete(input)
}

pub fn var_or_num(input: Span) -> IResult<Span, VarNum> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a VarNum:{}", input);
    }
    alt((into(num), into(var_name), into(sep_expr))).parse_complete(input)
}

pub fn var_or_val(input: Span) -> IResult<Span, VarOrVal> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a VarOrVal:{}", input);
    }
    alt((
        into(num),
        into(var_name),
        into(range),
        into(list),
        into(funcs),
        into(expr),
        into(sep_expr),
    ))
    .parse_complete(input)
}

pub fn variable(input: Span) -> IResult<Span, Variable> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a variable assignment:{}", input);
    }
    let (input, name) = space_around(var_name).parse_complete(input)?;
    let (input, _) = space_around(assign).parse_complete(input)?;
    let (input, value) = var_or_val(input)?;
    Ok((input, Variable::new(name, value)))
}

pub fn empty_line(input: Span) -> IResult<Span, Line> {
    let (input, _) = opt_multispace0.parse_complete(input)?;
    if input.fragment().split_whitespace().all(|s| s.is_empty()) {
        Ok((input, Line::Empty))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::NonEmpty,
        )))
    }
}

pub fn line(input: Span) -> IResult<Span, Line> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a line:{}", input);
    }
    terminated(
        alt((
            empty_line,
            into(variable),
            into(compare),
            into(funcs),
            into(expr),
            into(loops),
        )),
        newline,
    )
    .parse_complete(input)
}

pub fn lines(input: Span) -> IResult<Span, VecDeque<Line>> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for many lines:{}", input);
    }
    terminated(into(many1(line)), eof).parse_complete(input)
}

#[cfg(test)]
mod test {
    use crate::bitops::BitOps;
    use crate::parsers::ast::{BitExpr, Funcs, Number, SepBitExpr};

    use super::*;

    //     #[test]
    //     fn test_lines() {
    //         unsafe {
    //             assert_eq!(
    //                 lines(Span::new(
    //                     r#"
    // var = [0, 1, 1, 0, 0, 1];
    // n = 0;
    // for (i in var) {
    //     n = n >> 1;
    //     if (i == 6) {
    //         n = n | 0x800;
    //     }
    // }
    //         "#
    //                 )),
    //                 todo!()
    //             )
    //         }
    //     }

    #[test]
    fn test_assign() {
        unsafe {
            assert_eq!(
                assign(Span::new("= 8")),
                Ok((Span::new_from_raw_offset(1, 1, " 8", ()), '='))
            )
        }
    }

    #[test]
    fn test_variable() {
        unsafe {
            assert_eq!(
                var_name(Span::new("someValue_thatIsGood = 0x1")),
                Ok((
                    Span::new_from_raw_offset(20, 1, " = 0x1", ()),
                    Span::new_from_raw_offset(0, 1, "someValue_thatIsGood", ())
                ))
            )
        }
    }

    #[test]
    fn test_variable_assignment() {
        unsafe {
            assert_eq!(
                variable(Span::new("someValue_thatIsGood = 0x1 ")),
                Ok((
                    Span::new_from_raw_offset(26, 1, " ", ()),
                    Variable::new(
                        Span::new_from_raw_offset(0, 1, "someValue_thatIsGood", ()),
                        VarOrVal::Num(Number::new(
                            1,
                            Span::new_from_raw_offset(25, 1, "1", ()),
                            Some(Span::new_from_raw_offset(23, 1, "0x", ()))
                        ))
                    )
                ))
            )
        }
    }

    #[test]
    fn test_complex_variable_assignment() {
        unsafe {
            assert_eq!(
                variable(Span::new("someValue = ( 0x1 >> 1 ) | 0x800 ")),
                Ok((
                    Span::new_from_raw_offset(32, 1, " ", ()),
                    Variable::new(
                        Span::new_from_raw_offset(0, 1, "someValue", ()),
                        VarOrVal::Expr(BitExpr::new(
                            VarNum::Expr(Box::new(SepBitExpr::new(
                                Span::new_from_raw_offset(12, 1, "(", ()),
                                BitExpr::new(
                                    VarNum::Num(Number::new(
                                        1,
                                        Span::new_from_raw_offset(16, 1, "1", ()),
                                        Some(Span::new_from_raw_offset(14, 1, "0x", ()))
                                    )),
                                    BitOps::RightShift,
                                    Span::new_from_raw_offset(18, 1, ">>", ()),
                                    Some(VarNum::Num(Number::new(
                                        1,
                                        Span::new_from_raw_offset(21, 1, "1", ()),
                                        None
                                    )))
                                ),
                                Span::new_from_raw_offset(23, 1, ")", ())
                            ))),
                            BitOps::Or,
                            Span::new_from_raw_offset(25, 1, "|", ()),
                            Some(VarNum::Num(Number::new(
                                2048,
                                Span::new_from_raw_offset(29, 1, "800", ()),
                                Some(Span::new_from_raw_offset(27, 1, "0x", ()))
                            )))
                        ))
                    )
                ))
            )
        }
    }

    #[test]
    fn test_lines() {
        unsafe {
            let mut l = VecDeque::new();
            l.push_back(Line::Variable(Variable::new(
                Span::new_from_raw_offset(1, 2, "i", ()),
                VarOrVal::Num(Number::new(
                    2048,
                    Span::new_from_raw_offset(7, 2, "800", ()),
                    Some(Span::new_from_raw_offset(5, 2, "0x", ())),
                )),
            )));
            l.push_back(Line::Variable(Variable::new(
                Span::new_from_raw_offset(11, 3, "j", ()),
                VarOrVal::Num(Number::new(
                    400,
                    Span::new_from_raw_offset(15, 3, "400", ()),
                    None,
                )),
            )));
            l.push_back(Line::Expr(BitExpr::new(
                VarNum::Expr(Box::new(SepBitExpr::new(
                    Span::new_from_raw_offset(19, 4, "(", ()),
                    BitExpr::new(
                        VarNum::Var(Span::new_from_raw_offset(21, 4, "i", ())),
                        BitOps::RightShift,
                        Span::new_from_raw_offset(23, 4, ">>", ()),
                        Some(VarNum::Var(Span::new_from_raw_offset(26, 4, "j", ()))),
                    ),
                    Span::new_from_raw_offset(28, 4, ")", ()),
                ))),
                BitOps::Or,
                Span::new_from_raw_offset(30, 4, "|", ()),
                Some(VarNum::Num(Number::new(
                    42,
                    Span::new_from_raw_offset(34, 4, "101010", ()),
                    Some(Span::new_from_raw_offset(32, 4, "0b", ())),
                ))),
            )));
            l.push_back(Line::Func(Funcs::Quit(Span::new_from_raw_offset(
                41,
                5,
                "exit",
                (),
            ))));
            let test = lines(Span::new(
                r#"
i = 0x800
j = 400
( i >> j ) | 0b101010
exit()
"#,
            ));
            assert_eq!(test, Ok((Span::new_from_raw_offset(48, 6, "", ()), l)))
        }
    }
}
