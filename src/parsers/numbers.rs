use super::{Span, ast::Number, utils::opt_multispace0};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, take_while},
    combinator::map_res,
    sequence::preceded,
};

fn from_hex(input: Span) -> Result<(usize, Span), std::num::ParseIntError> {
    usize::from_str_radix(input.fragment(), 16).map(|out| (out, input))
}

fn from_octal(input: Span) -> Result<(usize, Span), std::num::ParseIntError> {
    usize::from_str_radix(input.fragment(), 8).map(|out| (out, input))
}

fn from_binary(input: Span) -> Result<(usize, Span), std::num::ParseIntError> {
    usize::from_str_radix(input.fragment(), 2).map(|out| (out, input))
}

fn from_decimal(input: Span) -> Result<(usize, Span), std::num::ParseIntError> {
    input.fragment().parse::<usize>().map(|out| (out, input))
}

fn is_hex_digit(c: char) -> bool {
    c.is_ascii_hexdigit()
}

fn is_bin_digit(c: char) -> bool {
    c.is_digit(2)
}

fn is_oct_digit(c: char) -> bool {
    c.is_digit(8)
}

fn is_dec_digit(c: char) -> bool {
    c.is_ascii_digit()
}

fn get_hex_num(input: Span) -> IResult<Span, (usize, Span)> {
    map_res(take_while(is_hex_digit), from_hex).parse_complete(input)
}

fn get_oct_num(input: Span) -> IResult<Span, (usize, Span)> {
    map_res(take_while(is_oct_digit), from_octal).parse_complete(input)
}

fn get_bin_num(input: Span) -> IResult<Span, (usize, Span)> {
    map_res(take_while(is_bin_digit), from_binary).parse_complete(input)
}

fn get_dec_num(input: Span) -> IResult<Span, (usize, Span)> {
    map_res(take_while(is_dec_digit), from_decimal).parse_complete(input)
}

fn hex_tag(input: Span) -> IResult<Span, Span> {
    alt((tag("0x"), tag("0X"))).parse_complete(input)
}

fn oct_tag(input: Span) -> IResult<Span, Span> {
    alt((tag("0o"), tag("0O"))).parse_complete(input)
}

fn bin_tag(input: Span) -> IResult<Span, Span> {
    alt((tag("0b"), tag("0B"))).parse_complete(input)
}

pub fn hex_num(input: Span) -> IResult<Span, Number> {
    let (input, span1) = hex_tag(input)?;
    let (input, (number, span2)) = get_hex_num(input)?;
    Ok((input, Number::new(number, span2, Some(span1))))
}

pub fn oct_num(input: Span) -> IResult<Span, Number> {
    let (input, span1) = oct_tag(input)?;
    let (input, (number, span2)) = (get_oct_num).parse_complete(input)?;
    Ok((input, Number::new(number, span2, Some(span1))))
}

pub fn bin_num(input: Span) -> IResult<Span, Number> {
    let (input, span1) = bin_tag(input)?;
    let (input, (number, span2)) = (get_bin_num).parse_complete(input)?;
    Ok((input, Number::new(number, span2, Some(span1))))
}

pub fn dec_num(input: Span) -> IResult<Span, Number> {
    let (input, (number, span)) = (get_dec_num).parse_complete(input)?;
    Ok((input, Number::new(number, span, None)))
}

pub fn num(input: Span) -> IResult<Span, Number> {
    let (input, number) = preceded(opt_multispace0, alt((hex_num, oct_num, bin_num, dec_num)))
        .parse_complete(input)?;
    Ok((input, number))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_decimal_number() {
        unsafe {
            assert_eq!(
                num(Span::new("1 ) | 0x800")),
                Ok((
                    Span::new_from_raw_offset(1, 1, " ) | 0x800", ()),
                    Number::new(1, Span::new_from_raw_offset(0, 1, "1", ()), None)
                ))
            );
            assert_eq!(
                dec_num(Span::new("10 ")),
                Ok((
                    Span::new_from_raw_offset(2, 1, " ", ()),
                    Number::new(10, Span::new_from_raw_offset(0, 1, "10", ()), None)
                ))
            );
            assert_eq!(
                num(Span::new("10 ")),
                Ok((
                    Span::new_from_raw_offset(2, 1, " ", ()),
                    Number::new(10, Span::new_from_raw_offset(0, 1, "10", ()), None)
                ))
            );
        }
    }

    #[test]
    fn test_parse_hexadecimal_number() {
        unsafe {
            assert_eq!(
                hex_num(Span::new("0x10 ")),
                Ok((
                    Span::new_from_raw_offset(4, 1, " ", ()),
                    Number::new(
                        16,
                        Span::new_from_raw_offset(2, 1, "10", ()),
                        Some(Span::new_from_raw_offset(0, 1, "0x", ()))
                    )
                ))
            );
            assert_eq!(
                num(Span::new("0x10 ")),
                Ok((
                    Span::new_from_raw_offset(4, 1, " ", ()),
                    Number::new(
                        16,
                        Span::new_from_raw_offset(2, 1, "10", ()),
                        Some(Span::new_from_raw_offset(0, 1, "0x", ()))
                    )
                ))
            );
        }
    }

    #[test]
    fn test_parse_octal_number() {
        unsafe {
            assert_eq!(
                oct_num(Span::new("0o10 ")),
                Ok((
                    Span::new_from_raw_offset(4, 1, " ", ()),
                    Number::new(
                        8,
                        Span::new_from_raw_offset(2, 1, "10", ()),
                        Some(Span::new_from_raw_offset(0, 1, "0o", ()))
                    )
                ))
            );
            assert_eq!(
                num(Span::new("0o10 ")),
                Ok((
                    Span::new_from_raw_offset(4, 1, " ", ()),
                    Number::new(
                        8,
                        Span::new_from_raw_offset(2, 1, "10", ()),
                        Some(Span::new_from_raw_offset(0, 1, "0o", ()))
                    )
                ))
            );
        }
    }

    #[test]
    fn test_parse_binary_number() {
        unsafe {
            assert_eq!(
                bin_num(Span::new("0b10 ")),
                Ok((
                    Span::new_from_raw_offset(4, 1, " ", ()),
                    Number::new(
                        2,
                        Span::new_from_raw_offset(2, 1, "10", ()),
                        Some(Span::new_from_raw_offset(0, 1, "0b", ()))
                    )
                ))
            );
            assert_eq!(
                num(Span::new("0b10 ")),
                Ok((
                    Span::new_from_raw_offset(4, 1, " ", ()),
                    Number::new(
                        2,
                        Span::new_from_raw_offset(2, 1, "10", ()),
                        Some(Span::new_from_raw_offset(0, 1, "0b", ()))
                    )
                ))
            );
        }
    }
}
