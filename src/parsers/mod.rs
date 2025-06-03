pub mod ast;
pub mod bitops;
pub mod compare;
pub mod exprs;
pub mod funcs;
pub mod general;
pub mod loops;
pub mod numbers;
pub mod utils;

use self::exprs::NumOrList;
use crate::repl::help::{EW, NE, NS, SE};
use color_print::{cformat, cwriteln};
use nom_locate::LocatedSpan;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::Mutex;

pub static VARIABLES: Mutex<BTreeMap<String, NumOrList>> = Mutex::new(BTreeMap::new());

pub type Span<'a> = LocatedSpan<&'a str>;

pub enum XodErr<I> {
    Parse(I, nom::error::ErrorKind),
    Incomplete(I, nom::Needed),
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ExprError<'a> {
    #[error("{0}")]
    Partial(PartialEvalError<'a>),
    #[error("Exiting...")]
    Quit,
    #[error("Help requested.")]
    Help,
    #[error("Printing history...")]
    History,
    #[error("Clearing screen...")]
    Clear,
}

impl<'a> From<PartialEvalError<'a>> for ExprError<'a> {
    fn from(value: PartialEvalError<'a>) -> Self {
        Self::Partial(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("Error parsing {loc:#?}. {msg} {fix}")]
pub struct PartialEvalError<'a> {
    pub loc: Span<'a>,
    pub msg: String,
    pub fix: String,
}

impl<'a> From<(PartialEvalError<'a>, Span<'a>)> for EvalError<'a> {
    fn from(value: (PartialEvalError<'a>, Span<'a>)) -> Self {
        Self {
            msg: value.0.msg,
            loc: value.0.loc,
            body: value.1,
            fix: value.0.fix,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub struct EvalError<'a> {
    pub msg: String,
    pub loc: Span<'a>,
    pub body: Span<'a>,
    pub fix: String,
}

impl fmt::Display for EvalError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        cwriteln!(f, "<s><r!>error</>: {}</>", self.msg)?;
        let start = self.loc.naive_get_utf8_column();
        let end = self.loc.fragment().len();
        let line = self.loc.location_line() as usize - 1;
        cwriteln!(
            f,
            "   <s><b!>{SE}{EW}></> line {line}, cols {start}-{}</>",
            start + end,
        )?;
        let sep_line = cformat!("<s><b!>{NS}</></>");
        writeln!(f, "   {sep_line}")?;
        let space = vec![' '; start - 1].into_iter().collect::<String>();
        let underline = cformat!("<s><c!>{}</></>", vec!['^'; end].iter().collect::<String>());
        let arrow1 = cformat!("<s><c!>{NS}</></>");
        for (i, b) in self.body.split('\n').enumerate() {
            cwriteln!(f, "<s><b!>{i: >2}</> {sep_line}\t{b}")?;
            if i == line {
                writeln!(f, "   {sep_line}\t{space}{underline}")?;
                writeln!(f, "   {sep_line}\t{space}{arrow1}")?;
                cwriteln!(
                    f,
                    "   {sep_line}\t{space}<s><c!>{NE}{EW}</> Suggested fix:</> <g!>{}</>",
                    self.fix
                )?;
            }
        }
        Ok(())
    }
}

pub trait Expression<'a, 'b, T>
where
    'a: 'b,
    T: 'a,
{
    fn eval(&'b mut self) -> Result<T, ExprError<'a>>;
}

pub trait RefSpan<'a> {
    fn get_span<'b>(&self) -> Span<'b>
    where
        'a: 'b;
}

impl<'a> RefSpan<'a> for Span<'a> {
    fn get_span<'b>(&self) -> Span<'b>
    where
        'a: 'b,
    {
        *self
    }
}
