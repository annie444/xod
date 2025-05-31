use super::{DEBUG_PRINT, Span};
use nom::{
    IResult, Parser, bytes::complete::tag, character::streaming::multispace0, sequence::delimited,
};

#[inline(always)]
pub fn open_bracket(input: Span) -> IResult<Span, Span> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for an open bracket:{}", input);
    }
    delimited(multispace0, tag("["), multispace0).parse(input)
}

#[inline(always)]
pub fn close_bracket(input: Span) -> IResult<Span, Span> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a close bracket:{}", input);
    }
    delimited(multispace0, tag("]"), multispace0).parse(input)
}

#[inline(always)]
pub fn open_paren(input: Span) -> IResult<Span, Span> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for an open parenthese:{}", input);
    }
    delimited(multispace0, tag("("), multispace0).parse(input)
}

#[inline(always)]
pub fn close_paren(input: Span) -> IResult<Span, Span> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a close parenthese:{}", input);
    }
    delimited(multispace0, tag(")"), multispace0).parse(input)
}

#[inline(always)]
pub fn open_brace(input: Span) -> IResult<Span, Span> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for an open curly brace:{}", input);
    }
    delimited(multispace0, tag("{"), multispace0).parse(input)
}

#[inline(always)]
pub fn close_brace(input: Span) -> IResult<Span, Span> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a close curly brace:{}", input);
    }
    delimited(multispace0, tag("}"), multispace0).parse(input)
}

#[inline(always)]
pub fn comma(input: Span) -> IResult<Span, Span> {
    if *DEBUG_PRINT {
        eprintln!("Parsing input for a comma separator:{}", input);
    }
    delimited(multispace0, tag(","), multispace0).parse(input)
}
