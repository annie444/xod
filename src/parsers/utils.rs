use super::{DEBUG_PRINT, Span};
use nom::{
    IResult, Parser,
    bytes::complete::tag,
    character::streaming::multispace0,
    sequence::{delimited, preceded, terminated},
};

pub fn space_around<'a, P, O>(
    input: P,
) -> impl Parser<Span<'a>, Output = O, Error = nom::error::Error<Span<'a>>>
where
    P: Parser<Span<'a>, Output = O, Error = nom::error::Error<Span<'a>>>,
{
    delimited(opt_multispace0, input, opt_multispace0)
}

pub fn opt_multispace0(input: Span) -> IResult<Span, ()> {
    let input = match (multispace0).parse_complete(input) {
        Ok((input, _)) => input,
        Err(e) => match e {
            nom::Err::Incomplete(_) => input,
            nom::Err::Error(_) => input,
            nom::Err::Failure(_) => return Err(e),
        },
    };
    Ok((input, ()))
}

#[inline(always)]
pub fn open_bracket(input: Span) -> IResult<Span, Span> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for an open bracket:{}", input);
    }
    terminated(tag("["), opt_multispace0).parse_complete(input)
}

#[inline(always)]
pub fn close_bracket(input: Span) -> IResult<Span, Span> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a close bracket:{}", input);
    }
    preceded(opt_multispace0, tag("]")).parse_complete(input)
}

#[inline(always)]
pub fn open_paren(input: Span) -> IResult<Span, Span> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for an open parenthese:{}", input);
    }
    terminated(tag("("), opt_multispace0).parse_complete(input)
}

#[inline(always)]
pub fn close_paren(input: Span) -> IResult<Span, Span> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a close parenthese:{}", input);
    }
    preceded(opt_multispace0, tag(")")).parse_complete(input)
}

#[inline(always)]
pub fn open_brace(input: Span) -> IResult<Span, Span> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for an open curly brace:{}", input);
    }
    space_around(tag("{")).parse_complete(input)
}

#[inline(always)]
pub fn close_brace(input: Span) -> IResult<Span, Span> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a close curly brace:{}", input);
    }
    space_around(tag("}")).parse_complete(input)
}

#[inline(always)]
pub fn comma(input: Span) -> IResult<Span, Span> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a comma separator:{}", input);
    }
    terminated(tag(","), opt_multispace0).parse_complete(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_brackets() {
        let input = Span::new("[ ]");
        let result1 = (open_bracket).parse_complete(input);
        eprintln!("{:?}", result1);
        assert!(result1.is_ok());
        let (input, result1) = result1.unwrap();
        let result2 = (close_bracket).parse_complete(input);
        eprintln!("{:?}", result2);
        assert!(result2.is_ok());
        let (_, result2) = result2.unwrap();
        unsafe {
            assert_eq!(result1, Span::new_from_raw_offset(0, 1, "[", ()));
            assert_eq!(result2, Span::new_from_raw_offset(2, 1, "]", ()))
        }
    }

    #[test]
    fn test_braces() {
        let input = Span::new("{ }");
        let result1 = (open_brace).parse_complete(input);
        eprintln!("{:?}", result1);
        assert!(result1.is_ok());
        let (input, result1) = result1.unwrap();
        let result2 = (close_brace).parse_complete(input);
        eprintln!("{:?}", result2);
        assert!(result2.is_ok());
        let (_, result2) = result2.unwrap();
        unsafe {
            assert_eq!(result1, Span::new_from_raw_offset(0, 1, "{", ()));
            assert_eq!(result2, Span::new_from_raw_offset(2, 1, "}", ()))
        }
    }

    #[test]
    fn test_parentheses() {
        let input = Span::new("( )");
        let result1 = (open_paren).parse_complete(input);
        eprintln!("{:?}", result1);
        assert!(result1.is_ok());
        let (input, result1) = result1.unwrap();
        let result2 = (close_paren).parse_complete(input);
        eprintln!("{:?}", result2);
        assert!(result2.is_ok());
        let (_, result2) = result2.unwrap();
        unsafe {
            assert_eq!(result1, Span::new_from_raw_offset(0, 1, "(", ()));
            assert_eq!(result2, Span::new_from_raw_offset(2, 1, ")", ()))
        }
    }
}
