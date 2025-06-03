use super::{
    Span,
    ast::{BitExpr, SepBitExpr},
    general::var_or_num,
    utils::{close_paren, open_paren, opt_multispace0},
};
use crate::bitops::BitOps;
use nom::{
    IResult, Parser, branch::alt, bytes::complete::tag, character::complete::multispace0,
    sequence::terminated,
};

pub fn bit_or(input: Span) -> IResult<Span, (BitOps, Span)> {
    let (input, op) = tag("|").parse_complete(input)?;
    Ok((input, (BitOps::Or, op)))
}

pub fn bit_xor(input: Span) -> IResult<Span, (BitOps, Span)> {
    let (input, op) = tag("^").parse_complete(input)?;
    Ok((input, (BitOps::Xor, op)))
}

pub fn bit_and(input: Span) -> IResult<Span, (BitOps, Span)> {
    let (input, op) = tag("&").parse_complete(input)?;
    Ok((input, (BitOps::And, op)))
}

pub fn bit_not(input: Span) -> IResult<Span, (BitOps, Span)> {
    let (input, op) = alt((tag("!"), tag("~"))).parse_complete(input)?;
    Ok((input, (BitOps::Not, op)))
}

pub fn bit_left(input: Span) -> IResult<Span, (BitOps, Span)> {
    let (input, op) = tag("<<").parse_complete(input)?;
    Ok((input, (BitOps::LeftShift, op)))
}

pub fn bit_right(input: Span) -> IResult<Span, (BitOps, Span)> {
    let (input, op) = tag(">>").parse_complete(input)?;
    Ok((input, (BitOps::RightShift, op)))
}

pub fn dual_bit_ops(input: Span) -> IResult<Span, (BitOps, Span)> {
    alt((bit_or, bit_xor, bit_and, bit_left, bit_right)).parse_complete(input)
}

pub fn dual_expr(input: Span) -> IResult<Span, BitExpr> {
    let (input, left) = terminated(var_or_num, opt_multispace0).parse_complete(input)?;
    let (input, (op, op_span)) = terminated(dual_bit_ops, opt_multispace0).parse_complete(input)?;
    let (input, right) = var_or_num(input)?;
    Ok((input, BitExpr::new(left, op, op_span, Some(right))))
}

pub fn single_expr(input: Span) -> IResult<Span, BitExpr> {
    let (input, (op, op_span)) = terminated(bit_not, opt_multispace0).parse_complete(input)?;
    let (input, left) = var_or_num(input)?;
    Ok((input, BitExpr::new(left, op, op_span, None)))
}

pub fn expr(input: Span) -> IResult<Span, BitExpr> {
    let (input, _) = multispace0(input)?;
    alt((dual_expr, single_expr)).parse_complete(input)
}

pub fn sep_expr(input: Span) -> IResult<Span, SepBitExpr> {
    let (input, open) = open_paren(input)?;
    let (input, expr) = expr(input)?;
    let (input, close) = close_paren(input)?;
    Ok((input, SepBitExpr::new(open, expr, close)))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parsers::ast::{Number, VarNum};

    #[test]
    fn test_seperated_expression() {
        unsafe {
            assert_eq!(
                sep_expr(Span::new("(0x1 >> 1) ")),
                Ok((
                    Span::new_from_raw_offset(10, 1, " ", ()),
                    SepBitExpr::new(
                        Span::new_from_raw_offset(0, 1, "(", ()),
                        BitExpr::new(
                            VarNum::Num(Number::new(
                                1,
                                Span::new_from_raw_offset(3, 1, "1", ()),
                                Some(Span::new_from_raw_offset(1, 1, "0x", ()))
                            )),
                            BitOps::RightShift,
                            Span::new_from_raw_offset(5, 1, ">>", ()),
                            Some(VarNum::Num(Number::new(
                                1,
                                Span::new_from_raw_offset(8, 1, "1", ()),
                                None
                            )))
                        ),
                        Span::new_from_raw_offset(9, 1, ")", ())
                    )
                ))
            )
        }
    }

    #[test]
    fn test_bitwise_or() {
        unsafe {
            assert_eq!(
                bit_or(Span::new("| 8")),
                Ok((
                    Span::new_from_raw_offset(1, 1, " 8", ()),
                    (BitOps::Or, Span::new_from_raw_offset(0, 1, "|", ()))
                ))
            );
            assert_eq!(
                dual_expr(Span::new("10 | 8 ")),
                Ok((
                    Span::new_from_raw_offset(6, 1, " ", ()),
                    BitExpr::new(
                        VarNum::Num(Number::new(
                            10,
                            Span::new_from_raw_offset(0, 1, "10", ()),
                            None
                        )),
                        BitOps::Or,
                        Span::new_from_raw_offset(3, 1, "|", ()),
                        Some(VarNum::Num(Number::new(
                            8,
                            Span::new_from_raw_offset(5, 1, "8", ()),
                            None
                        )))
                    )
                ))
            );
            assert_eq!(
                expr(Span::new("10 | 8 ")),
                Ok((
                    Span::new_from_raw_offset(6, 1, " ", ()),
                    BitExpr::new(
                        VarNum::Num(Number::new(
                            10,
                            Span::new_from_raw_offset(0, 1, "10", ()),
                            None
                        )),
                        BitOps::Or,
                        Span::new_from_raw_offset(3, 1, "|", ()),
                        Some(VarNum::Num(Number::new(
                            8,
                            Span::new_from_raw_offset(5, 1, "8", ()),
                            None
                        )))
                    )
                ))
            );
        }
    }

    #[test]
    fn test_bitwise_xor() {
        unsafe {
            assert_eq!(
                bit_xor(Span::new("^ 8")),
                Ok((
                    Span::new_from_raw_offset(1, 1, " 8", ()),
                    (BitOps::Xor, Span::new_from_raw_offset(0, 1, "^", ()))
                ))
            );
            assert_eq!(
                dual_expr(Span::new("10 ^ 8 ")),
                Ok((
                    Span::new_from_raw_offset(6, 1, " ", ()),
                    BitExpr::new(
                        VarNum::Num(Number::new(
                            10,
                            Span::new_from_raw_offset(0, 1, "10", ()),
                            None
                        )),
                        BitOps::Xor,
                        Span::new_from_raw_offset(3, 1, "^", ()),
                        Some(VarNum::Num(Number::new(
                            8,
                            Span::new_from_raw_offset(5, 1, "8", ()),
                            None
                        )))
                    )
                ))
            );
            assert_eq!(
                expr(Span::new("10 ^ 8 ")),
                Ok((
                    Span::new_from_raw_offset(6, 1, " ", ()),
                    BitExpr::new(
                        VarNum::Num(Number::new(
                            10,
                            Span::new_from_raw_offset(0, 1, "10", ()),
                            None
                        )),
                        BitOps::Xor,
                        Span::new_from_raw_offset(3, 1, "^", ()),
                        Some(VarNum::Num(Number::new(
                            8,
                            Span::new_from_raw_offset(5, 1, "8", ()),
                            None
                        )))
                    )
                ))
            );
        }
    }

    #[test]
    fn test_bitwise_and() {
        unsafe {
            assert_eq!(
                bit_and(Span::new("& 8")),
                Ok((
                    Span::new_from_raw_offset(1, 1, " 8", ()),
                    (BitOps::And, Span::new_from_raw_offset(0, 1, "&", ()))
                ))
            );
            assert_eq!(
                dual_expr(Span::new("10 & 8 ")),
                Ok((
                    Span::new_from_raw_offset(6, 1, " ", ()),
                    BitExpr::new(
                        VarNum::Num(Number::new(
                            10,
                            Span::new_from_raw_offset(0, 1, "10", ()),
                            None
                        )),
                        BitOps::And,
                        Span::new_from_raw_offset(3, 1, "&", ()),
                        Some(VarNum::Num(Number::new(
                            8,
                            Span::new_from_raw_offset(5, 1, "8", ()),
                            None
                        )))
                    )
                ))
            );
            assert_eq!(
                expr(Span::new("10 & 8 ")),
                Ok((
                    Span::new_from_raw_offset(6, 1, " ", ()),
                    BitExpr::new(
                        VarNum::Num(Number::new(
                            10,
                            Span::new_from_raw_offset(0, 1, "10", ()),
                            None
                        )),
                        BitOps::And,
                        Span::new_from_raw_offset(3, 1, "&", ()),
                        Some(VarNum::Num(Number::new(
                            8,
                            Span::new_from_raw_offset(5, 1, "8", ()),
                            None
                        )))
                    )
                ))
            );
        }
    }

    #[test]
    fn test_bitwise_not_1() {
        unsafe {
            assert_eq!(
                bit_not(Span::new("!8")),
                Ok((
                    Span::new_from_raw_offset(1, 1, "8", ()),
                    (BitOps::Not, Span::new_from_raw_offset(0, 1, "!", ()))
                ))
            );
            assert_eq!(
                single_expr(Span::new("!8 ")),
                Ok((
                    Span::new_from_raw_offset(2, 1, " ", ()),
                    BitExpr::new(
                        VarNum::Num(Number::new(
                            8,
                            Span::new_from_raw_offset(1, 1, "8", ()),
                            None
                        )),
                        BitOps::Not,
                        Span::new_from_raw_offset(0, 1, "!", ()),
                        None
                    )
                ))
            );
            assert_eq!(
                expr(Span::new("!8 ")),
                Ok((
                    Span::new_from_raw_offset(2, 1, " ", ()),
                    BitExpr::new(
                        VarNum::Num(Number::new(
                            8,
                            Span::new_from_raw_offset(1, 1, "8", ()),
                            None
                        )),
                        BitOps::Not,
                        Span::new_from_raw_offset(0, 1, "!", ()),
                        None
                    )
                ))
            );
        }
    }

    #[test]
    fn test_bitwise_not_2() {
        unsafe {
            assert_eq!(
                bit_not(Span::new("~8")),
                Ok((
                    Span::new_from_raw_offset(1, 1, "8", ()),
                    (BitOps::Not, Span::new_from_raw_offset(0, 1, "~", ()))
                ))
            );
            assert_eq!(
                single_expr(Span::new("~8 ")),
                Ok((
                    Span::new_from_raw_offset(2, 1, " ", ()),
                    BitExpr::new(
                        VarNum::Num(Number::new(
                            8,
                            Span::new_from_raw_offset(1, 1, "8", ()),
                            None
                        )),
                        BitOps::Not,
                        Span::new_from_raw_offset(0, 1, "~", ()),
                        None
                    )
                ))
            );
            assert_eq!(
                expr(Span::new("~8 ")),
                Ok((
                    Span::new_from_raw_offset(2, 1, " ", ()),
                    BitExpr::new(
                        VarNum::Num(Number::new(
                            8,
                            Span::new_from_raw_offset(1, 1, "8", ()),
                            None
                        )),
                        BitOps::Not,
                        Span::new_from_raw_offset(0, 1, "~", ()),
                        None
                    )
                ))
            );
        }
    }

    #[test]
    fn test_bitwise_left_shift() {
        unsafe {
            assert_eq!(
                bit_left(Span::new("<< 8")),
                Ok((
                    Span::new_from_raw_offset(2, 1, " 8", ()),
                    (BitOps::LeftShift, Span::new_from_raw_offset(0, 1, "<<", ()))
                ))
            );
            assert_eq!(
                dual_expr(Span::new("10 << 8 ")),
                Ok((
                    Span::new_from_raw_offset(7, 1, " ", ()),
                    BitExpr::new(
                        VarNum::Num(Number::new(
                            10,
                            Span::new_from_raw_offset(0, 1, "10", ()),
                            None
                        )),
                        BitOps::LeftShift,
                        Span::new_from_raw_offset(3, 1, "<<", ()),
                        Some(VarNum::Num(Number::new(
                            8,
                            Span::new_from_raw_offset(6, 1, "8", ()),
                            None
                        )))
                    )
                ))
            );
            assert_eq!(
                expr(Span::new("10 << 8 ")),
                Ok((
                    Span::new_from_raw_offset(7, 1, " ", ()),
                    BitExpr::new(
                        VarNum::Num(Number::new(
                            10,
                            Span::new_from_raw_offset(0, 1, "10", ()),
                            None
                        )),
                        BitOps::LeftShift,
                        Span::new_from_raw_offset(3, 1, "<<", ()),
                        Some(VarNum::Num(Number::new(
                            8,
                            Span::new_from_raw_offset(6, 1, "8", ()),
                            None
                        )))
                    )
                ))
            );
        }
    }

    #[test]
    fn test_bitwise_right_shift() {
        unsafe {
            assert_eq!(
                bit_right(Span::new(">> 8")),
                Ok((
                    Span::new_from_raw_offset(2, 1, " 8", ()),
                    (
                        BitOps::RightShift,
                        Span::new_from_raw_offset(0, 1, ">>", ())
                    )
                ))
            );
            assert_eq!(
                dual_expr(Span::new("10 >> 8 ")),
                Ok((
                    Span::new_from_raw_offset(7, 1, " ", ()),
                    BitExpr::new(
                        VarNum::Num(Number::new(
                            10,
                            Span::new_from_raw_offset(0, 1, "10", ()),
                            None
                        )),
                        BitOps::RightShift,
                        Span::new_from_raw_offset(3, 1, ">>", ()),
                        Some(VarNum::Num(Number::new(
                            8,
                            Span::new_from_raw_offset(6, 1, "8", ()),
                            None
                        )))
                    )
                ))
            );
            assert_eq!(
                expr(Span::new("10 >> 8 ")),
                Ok((
                    Span::new_from_raw_offset(7, 1, " ", ()),
                    BitExpr::new(
                        VarNum::Num(Number::new(
                            10,
                            Span::new_from_raw_offset(0, 1, "10", ()),
                            None
                        )),
                        BitOps::RightShift,
                        Span::new_from_raw_offset(3, 1, ">>", ()),
                        Some(VarNum::Num(Number::new(
                            8,
                            Span::new_from_raw_offset(6, 1, "8", ()),
                            None
                        )))
                    )
                ))
            );
        }
    }
}
