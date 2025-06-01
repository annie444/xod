use super::{
    DEBUG_PRINT, Span,
    ast::{Compare, CompareOp},
    general::var_or_num,
    utils::space_around,
};
use nom::{IResult, Parser, branch::alt, bytes::complete::tag};

pub fn equals(input: Span) -> IResult<Span, (Compare, Span)> {
    let (input, op) = tag("==").parse_complete(input)?;
    Ok((input, (Compare::Equal, op)))
}

pub fn not_equals(input: Span) -> IResult<Span, (Compare, Span)> {
    let (input, op) = tag("!=").parse_complete(input)?;
    Ok((input, (Compare::NotEqual, op)))
}

pub fn greater(input: Span) -> IResult<Span, (Compare, Span)> {
    let (input, op) = tag(">").parse_complete(input)?;
    Ok((input, (Compare::Greater, op)))
}

pub fn greater_equal(input: Span) -> IResult<Span, (Compare, Span)> {
    let (input, op) = tag(">=").parse_complete(input)?;
    Ok((input, (Compare::GreaterEqual, op)))
}

pub fn lesser(input: Span) -> IResult<Span, (Compare, Span)> {
    let (input, op) = tag("<").parse_complete(input)?;
    Ok((input, (Compare::Less, op)))
}

pub fn lesser_equal(input: Span) -> IResult<Span, (Compare, Span)> {
    let (input, op) = tag("<=").parse_complete(input)?;
    Ok((input, (Compare::LessEqual, op)))
}

pub fn operator(input: Span) -> IResult<Span, (Compare, Span)> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a boolean operator:{}", input);
    }
    alt((
        equals,
        not_equals,
        greater_equal,
        lesser_equal,
        greater,
        lesser,
    ))
    .parse_complete(input)
}

pub fn compare(input: Span) -> IResult<Span, CompareOp> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a boolean expression:{}", input);
    }
    let (input, left) = space_around(var_or_num).parse_complete(input)?;
    let (input, (op, op_span)) = space_around(operator).parse_complete(input)?;
    let (input, right) = var_or_num(input)?;
    Ok((input, CompareOp::new(left, op, op_span, right)))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parsers::ast::{Number, VarNum};

    #[test]
    fn test_equals() {
        unsafe {
            assert_eq!(
                equals(Span::new("== 10")),
                Ok((
                    Span::new_from_raw_offset(2, 1, " 10", ()),
                    (Compare::Equal, Span::new_from_raw_offset(0, 1, "==", ()))
                ))
            );
            assert_eq!(
                operator(Span::new("== 10")),
                Ok((
                    Span::new_from_raw_offset(2, 1, " 10", ()),
                    (Compare::Equal, Span::new_from_raw_offset(0, 1, "==", ()))
                ))
            );
        }
    }

    #[test]
    fn test_not_equals() {
        unsafe {
            assert_eq!(
                not_equals(Span::new("!= 10")),
                Ok((
                    Span::new_from_raw_offset(2, 1, " 10", ()),
                    (Compare::NotEqual, Span::new_from_raw_offset(0, 1, "!=", ()))
                ))
            );
            assert_eq!(
                operator(Span::new("!= 10")),
                Ok((
                    Span::new_from_raw_offset(2, 1, " 10", ()),
                    (Compare::NotEqual, Span::new_from_raw_offset(0, 1, "!=", ()))
                ))
            );
        }
    }

    #[test]
    fn test_greater_than() {
        unsafe {
            assert_eq!(
                greater(Span::new("> 10")),
                Ok((
                    Span::new_from_raw_offset(1, 1, " 10", ()),
                    (Compare::Greater, Span::new_from_raw_offset(0, 1, ">", ()))
                ))
            );
            assert_eq!(
                operator(Span::new("> 10")),
                Ok((
                    Span::new_from_raw_offset(1, 1, " 10", ()),
                    (Compare::Greater, Span::new_from_raw_offset(0, 1, ">", ()))
                ))
            );
        }
    }

    #[test]
    fn test_greater_than_or_equal_to() {
        unsafe {
            assert_eq!(
                greater_equal(Span::new(">= 10")),
                Ok((
                    Span::new_from_raw_offset(2, 1, " 10", ()),
                    (
                        Compare::GreaterEqual,
                        Span::new_from_raw_offset(0, 1, ">=", ())
                    )
                ))
            );
            assert_eq!(
                operator(Span::new(">= 10")),
                Ok((
                    Span::new_from_raw_offset(2, 1, " 10", ()),
                    (
                        Compare::GreaterEqual,
                        Span::new_from_raw_offset(0, 1, ">=", ())
                    )
                ))
            );
        }
    }

    #[test]
    fn test_less_than() {
        unsafe {
            assert_eq!(
                lesser(Span::new("< 10")),
                Ok((
                    Span::new_from_raw_offset(1, 1, " 10", ()),
                    (Compare::Less, Span::new_from_raw_offset(0, 1, "<", ()))
                ))
            );
            assert_eq!(
                operator(Span::new("< 10")),
                Ok((
                    Span::new_from_raw_offset(1, 1, " 10", ()),
                    (Compare::Less, Span::new_from_raw_offset(0, 1, "<", ()))
                ))
            );
        }
    }

    #[test]
    fn test_less_than_or_equal_to() {
        unsafe {
            assert_eq!(
                lesser_equal(Span::new("<= 10")),
                Ok((
                    Span::new_from_raw_offset(2, 1, " 10", ()),
                    (
                        Compare::LessEqual,
                        Span::new_from_raw_offset(0, 1, "<=", ())
                    )
                ))
            );
            assert_eq!(
                operator(Span::new("<= 10")),
                Ok((
                    Span::new_from_raw_offset(2, 1, " 10", ()),
                    (
                        Compare::LessEqual,
                        Span::new_from_raw_offset(0, 1, "<=", ())
                    )
                ))
            );
        }
    }

    #[test]
    fn test_comparison() {
        unsafe {
            assert_eq!(
                compare(Span::new("10 >= 12 ")),
                Ok((
                    Span::new_from_raw_offset(8, 1, " ", ()),
                    CompareOp::new(
                        VarNum::Num(Number::new(
                            10,
                            Span::new_from_raw_offset(0, 1, "10", ()),
                            None
                        )),
                        Compare::GreaterEqual,
                        Span::new_from_raw_offset(3, 1, ">=", ()),
                        VarNum::Num(Number::new(
                            12,
                            Span::new_from_raw_offset(6, 1, "12", ()),
                            None
                        ))
                    )
                ))
            );

            assert_eq!(
                compare(Span::new("10 >= someVar ")),
                Ok((
                    Span::new_from_raw_offset(13, 1, " ", ()),
                    CompareOp::new(
                        VarNum::Num(Number::new(
                            10,
                            Span::new_from_raw_offset(0, 1, "10", ()),
                            None
                        )),
                        Compare::GreaterEqual,
                        Span::new_from_raw_offset(3, 1, ">=", ()),
                        VarNum::Var(Span::new_from_raw_offset(6, 1, "someVar", ()))
                    )
                ))
            );
        }
    }
}
