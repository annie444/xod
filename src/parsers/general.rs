use super::{
    Span,
    ast::{Line, Method, VarNum, VarOrVal, Variable},
    bitops::{expr, sep_expr},
    compare::compare,
    funcs::{funcs, range_func},
    loops::{list, loops},
    numbers::num,
    utils::{close_paren, open_paren, space_around},
};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, char, multispace0, space0},
    combinator::{eof, into, recognize},
    multi::{many0, many1},
    sequence::{delimited, pair, terminated},
};
use std::collections::VecDeque;

pub fn assign(input: Span) -> IResult<Span, char> {
    char('=').parse_complete(input)
}

pub fn var_name(input: Span) -> IResult<Span, Span> {
    recognize(pair(
        alt((alpha1, tag("_"))),
        many0(alt((alphanumeric1, tag("_")))),
    ))
    .parse_complete(input)
}

pub fn var_or_num(input: Span) -> IResult<Span, VarNum> {
    alt((
        into(num),
        into(funcs),
        into(method),
        into(var_name),
        into(sep_expr),
    ))
    .parse_complete(input)
}

pub fn var_or_val(input: Span) -> IResult<Span, VarOrVal> {
    alt((
        into(range_func),
        into(list),
        into(method),
        into(funcs),
        into(expr),
        into(sep_expr),
        into(num),
        into(var_name),
    ))
    .parse_complete(input)
}

pub fn variable(input: Span) -> IResult<Span, Variable> {
    let (input, name) = space_around(var_name).parse_complete(input)?;
    let (input, _) = space_around(assign).parse_complete(input)?;
    let (input, value) = var_or_val(input)?;
    Ok((input, Variable::new(name, value)))
}

pub fn empty_line(input: Span) -> IResult<Span, Line> {
    let (input, _) = many0(space0).parse_complete(input)?;
    let input_str = input.fragment();
    if input_str.split_whitespace().all(|s| s.is_empty()) {
        Ok((input, Line::Empty))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::NonEmpty,
        )))
    }
}

fn append(input: Span) -> IResult<Span, Method> {
    let (input, (var, method, arg)) = (
        var_name,
        delimited(char('.'), tag("append"), open_paren),
        terminated(var_or_num, close_paren),
    )
        .parse_complete(input)?;
    Ok((input, Method::Append(var, method, arg)))
}

fn prepend(input: Span) -> IResult<Span, Method> {
    let (input, (var, method, arg)) = (
        var_name,
        delimited(char('.'), tag("prepend"), open_paren),
        terminated(var_or_num, close_paren),
    )
        .parse_complete(input)?;
    Ok((input, Method::Prepend(var, method, arg)))
}

fn front(input: Span) -> IResult<Span, Method> {
    let (input, (var, method)) = (
        var_name,
        terminated(
            delimited(char('.'), alt((tag("front"), tag("pop"))), open_paren),
            close_paren,
        ),
    )
        .parse_complete(input)?;
    Ok((input, Method::Front(var, method)))
}

fn back(input: Span) -> IResult<Span, Method> {
    let (input, (var, method)) = (
        var_name,
        terminated(
            delimited(char('.'), alt((tag("back"), tag("pop_back"))), open_paren),
            close_paren,
        ),
    )
        .parse_complete(input)?;
    Ok((input, Method::Back(var, method)))
}

fn index(input: Span) -> IResult<Span, Method> {
    let (input, (var, method, arg)) = (
        var_name,
        delimited(char('.'), alt((tag("index"), tag("get"))), open_paren),
        terminated(var_or_num, close_paren),
    )
        .parse_complete(input)?;
    Ok((input, Method::Index(var, method, arg)))
}

pub fn method(input: Span) -> IResult<Span, Method> {
    alt((append, prepend, index, front, back)).parse_complete(input)
}

pub fn line(input: Span) -> IResult<Span, Line> {
    terminated(
        alt((
            empty_line,     // red
            into(method),   // yellow
            into(variable), // cyan
            into(compare),  // green
            into(funcs),    // blue
            into(expr),     // magenta
            into(loops),    // orange
        )),
        multispace0,
    )
    .parse_complete(input)
}

pub fn lines(input: Span) -> IResult<Span, VecDeque<Line>> {
    terminated(into(many1(line)), eof).parse_complete(input)
}

#[cfg(test)]
mod test {
    use crate::bitops::BitOps;
    use crate::parsers::ast::{
        BitExpr, Compare, CompareOp, Funcs, Iter, Loop, Loops, Number, SepBitExpr,
    };

    use super::*;

    #[test]
    fn test_methods() {
        let input = r#"
someList = [1, 2, 3, 4, 5]
someList.append(6)
someList.prepend(0)
a = someList.index(3)
b = someList.get(4)
c = someList.front()
e = someList.pop()
d = someList.back()
f = someList.pop_back()
"#;
        let span = Span::new(input);
        let result = lines(span);
        assert!(result.is_ok(), "Failed to parse methods: {:#?}", result);
        let (remaining, lines) = result.unwrap();
        assert_eq!(
            remaining.fragment(),
            &"",
            "Did not consume all input {:#?}",
            (remaining, lines)
        );
        assert_eq!(
            lines.len(),
            9,
            "Expected 7 lines, found {}: {:#?}",
            lines.len(),
            (remaining, lines)
        );

        // Check the first line
        if let Line::Variable(var) = &lines[0] {
            assert_eq!(
                var.name.fragment(),
                &"someList",
                "First variable name mismatch: {:#?}",
                var,
            );
            if let VarOrVal::List(list) = &var.value {
                assert_eq!(list.len(), 5, "Expected list of length 5: {:#?}", list);
            } else {
                panic!("Expected VarOrVal::List for someList: {:#?}", var);
            }
        } else {
            panic!(
                "Expected first line to be a Variable assignment, instead found: {:#?}",
                lines[0]
            );
        }

        // Check the second line
        if let Line::Method(method) = &lines[1] {
            assert!(
                matches!(method, Method::Append(_, _, _)),
                "Expected the append method: {:#?}",
                method,
            );
        } else {
            panic!(
                "Expected second line to be a method, instead found: {:#?}",
                lines[0]
            );
        }

        // Check the third line
        if let Line::Method(method) = &lines[2] {
            assert!(
                matches!(method, Method::Prepend(_, _, _)),
                "Expected the prepend method: {:#?}",
                method,
            );
        } else {
            panic!(
                "Expected second line to be a method, instead found: {:#?}",
                lines[0]
            );
        }

        // Check the fourth and fifth line
        for i in 3..=4 {
            if let Line::Variable(var) = &lines[i] {
                assert!(
                    var.name.fragment() == &"a" || var.name.fragment() == &"b",
                    "Expected variable `a` or `b`: {:#?}",
                    var,
                );
                if let VarOrVal::Method(method) = &var.value {
                    assert!(
                        matches!(method, Method::Index(_, _, _)),
                        "Expected the index method: {:#?}",
                        method,
                    );
                } else {
                    panic!(
                        "Expected VarOrVal::Method for assignment of variable `a` or `b`: {:#?}",
                        var
                    );
                }
            } else {
                panic!(
                    "Expected line to be a Variable assignment, instead found: {:#?}",
                    lines[i]
                );
            }
        }

        // Check the sixth and seventh line
        for i in 5..=6 {
            if let Line::Variable(var) = &lines[i] {
                assert!(
                    var.name.fragment() == &"c" || var.name.fragment() == &"e",
                    "Expected variable `c` or `e`: {:#?}",
                    var,
                );
                if let VarOrVal::Method(method) = &var.value {
                    assert!(
                        matches!(method, Method::Front(_, _)),
                        "Expected the front method: {:#?}",
                        method,
                    );
                } else {
                    panic!(
                        "Expected VarOrVal::Method for assignment of variable `c` or `e`: {:#?}",
                        var
                    );
                }
            } else {
                panic!(
                    "Expected line to be a Variable assignment, instead found: {:#?}",
                    lines[i]
                );
            }
        }

        // Check the eighth and ninth line
        for i in 7..=8 {
            if let Line::Variable(var) = &lines[i] {
                assert!(
                    var.name.fragment() == &"d" || var.name.fragment() == &"f",
                    "Expected variable `d` or `f`: {:#?}",
                    var,
                );
                if let VarOrVal::Method(method) = &var.value {
                    assert!(
                        matches!(method, Method::Back(_, _)),
                        "Expected the back method: {:#?}",
                        method,
                    );
                } else {
                    panic!(
                        "Expected VarOrVal::Method for assignment of variable `d` or `f`: {:#?}",
                        var
                    );
                }
            } else {
                panic!(
                    "Expected line to be a Variable assignment, instead found: {:#?}",
                    lines[i]
                );
            }
        }
    }

    #[test]
    fn test_nested_loops() {
        unsafe {
            let input = Span::new(
                r#"
var = [0, 1, 1, 0, 0, 1]
results = []
n = 0
for (i in var) {
    n = n >> 1
    if (i == 6) {
        n = n | 0x800
        results.append(hex(n))
    }
    hex(n)
}
"#,
            );
            let result = lines(input);
            assert!(result.is_ok(), "Failed to parse lines: {:?}", result);
            let result = result.unwrap();
            let mut lines = VecDeque::new();
            lines.push_back(Line::Variable(Variable::new(
                Span::new_from_raw_offset(1, 2, "var", ()),
                VarOrVal::List(
                    vec![
                        VarNum::Num(Number::new(
                            0,
                            Span::new_from_raw_offset(8, 2, "0", ()),
                            None,
                        )),
                        VarNum::Num(Number::new(
                            1,
                            Span::new_from_raw_offset(11, 2, "1", ()),
                            None,
                        )),
                        VarNum::Num(Number::new(
                            1,
                            Span::new_from_raw_offset(14, 2, "1", ()),
                            None,
                        )),
                        VarNum::Num(Number::new(
                            0,
                            Span::new_from_raw_offset(17, 2, "0", ()),
                            None,
                        )),
                        VarNum::Num(Number::new(
                            0,
                            Span::new_from_raw_offset(20, 2, "0", ()),
                            None,
                        )),
                        VarNum::Num(Number::new(
                            1,
                            Span::new_from_raw_offset(23, 2, "1", ()),
                            None,
                        )),
                    ]
                    .into(),
                ),
            )));
            lines.push_back(Line::Variable(Variable::new(
                Span::new_from_raw_offset(26, 3, "results", ()),
                VarOrVal::List(VecDeque::new()),
            )));
            lines.push_back(Line::Variable(Variable::new(
                Span::new_from_raw_offset(39, 4, "n", ()),
                VarOrVal::Num(Number::new(
                    0,
                    Span::new_from_raw_offset(43, 4, "0", ()),
                    None,
                )),
            )));
            let mut for_loop = Loop::new(
                Loops::For(
                    Span::new_from_raw_offset(45, 5, "for", ()),
                    Span::new_from_raw_offset(50, 5, "i", ()),
                    Iter::Var(Span::new_from_raw_offset(55, 5, "var", ())),
                ),
                Span::new_from_raw_offset(60, 5, "{", ()),
                Span::new_from_raw_offset(165, 12, "}", ()),
            );
            for_loop.add_line(Line::Variable(Variable::new(
                Span::new_from_raw_offset(66, 6, "n", ()),
                VarOrVal::Expr(BitExpr::new(
                    VarNum::Var(Span::new_from_raw_offset(70, 6, "n", ())),
                    BitOps::RightShift,
                    Span::new_from_raw_offset(72, 6, ">>", ()),
                    Some(VarNum::Num(Number::new(
                        1,
                        Span::new_from_raw_offset(75, 6, "1", ()),
                        None,
                    ))),
                )),
            )));
            let mut if_loop = Loop::new(
                Loops::If(
                    Span::new_from_raw_offset(81, 7, "if", ()),
                    CompareOp::new(
                        VarNum::Var(Span::new_from_raw_offset(85, 7, "i", ())),
                        Compare::Equal,
                        Span::new_from_raw_offset(87, 7, "==", ()),
                        VarNum::Num(Number::new(
                            6,
                            Span::new_from_raw_offset(90, 7, "6", ()),
                            None,
                        )),
                    ),
                ),
                Span::new_from_raw_offset(93, 7, "{", ()),
                Span::new_from_raw_offset(152, 10, "}", ()),
            );
            if_loop.add_line(Line::Variable(Variable::new(
                Span::new_from_raw_offset(103, 8, "n", ()),
                VarOrVal::Expr(BitExpr::new(
                    VarNum::Var(Span::new_from_raw_offset(107, 8, "n", ())),
                    BitOps::Or,
                    Span::new_from_raw_offset(109, 8, "|", ()),
                    Some(VarNum::Num(Number::new(
                        2048,
                        Span::new_from_raw_offset(113, 8, "800", ()),
                        Some(Span::new_from_raw_offset(111, 8, "0x", ())),
                    ))),
                )),
            )));
            if_loop.add_line(Line::Method(Method::Append(
                Span::new_from_raw_offset(125, 9, "results", ()),
                Span::new_from_raw_offset(133, 9, "append", ()),
                VarNum::Func(Box::new(Funcs::Hex(
                    Span::new_from_raw_offset(140, 9, "hex", ()),
                    VarNum::Var(Span::new_from_raw_offset(144, 9, "n", ())),
                ))),
            )));
            for_loop.add_line(Line::Loop(if_loop));
            for_loop.add_line(Line::Func(Funcs::Hex(
                Span::new_from_raw_offset(158, 11, "hex", ()),
                VarNum::Var(Span::new_from_raw_offset(162, 11, "n", ())),
            )));
            lines.push_back(Line::Loop(for_loop));
            assert_eq!(result, (Span::new_from_raw_offset(167, 13, "", ()), lines))
        }
    }

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

    #[test]
    fn test_list() {
        let line = r#"
set = [0x0, 0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8, 0x9, 0x10]
for(s in set) {
    hex(s)
    bin(s)
    oct(s)
}
"#;
        let span = Span::new(line);
        let result = lines(span);
        assert!(result.is_ok(), "Failed to parse list: {:?}", result);
    }
}
