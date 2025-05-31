pub mod ast;
pub mod bitops;
pub mod compare;
pub mod exprs;
pub mod funcs;
pub mod general;
pub mod loops;
pub mod numbers;
pub mod utils;

use color_print::{cformat, cwriteln};
use nom_locate::LocatedSpan;
use std::collections::BTreeMap;
use std::sync::Mutex;
use std::{fmt, option_env, sync::LazyLock};

pub static VARIABLES: Mutex<BTreeMap<&'static str, usize>> = Mutex::new(BTreeMap::new());

pub static DEBUG_PRINT: LazyLock<bool> =
    LazyLock::new(|| option_env!("PARSE_DEBUG").map(|_| true).unwrap_or(false));

pub type Span<'a> = LocatedSpan<&'a str>;

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
        let line = self.loc.location_line();
        cwriteln!(
            f,
            "<s><b!> --></> line {line}, cols {start}-{}</>",
            start + end,
        )?;
        let line = cformat!("<s><b!>|</></>");
        let body: String = self
            .body
            .split('\n')
            .enumerate()
            .map(|(i, s)| cformat!("\n{line} <b!>{i:0>2}</>\t{s}"))
            .collect();
        write!(f, "{line}")?;
        writeln!(f, "{body}")?;
        let space = vec![' '; start - 1].into_iter().collect::<String>();
        let underline = cformat!("<s><c!>{}</></>", vec!['^'; end].iter().collect::<String>());
        let arrow1 = cformat!("<s><c!>│</></>");
        let arrow2 = format!(
            "{}{arrow1}",
            vec![' '; end - 2].into_iter().collect::<String>(),
        );
        let underscore: String = cformat!(
            "<s><c!>└{}┘</></>",
            vec!['─'; end - 2].iter().collect::<String>()
        );
        writeln!(f, "{line}   \t{space}{underline}")?;
        writeln!(f, "    \t{space}{arrow1}{arrow2}")?;
        writeln!(f, "    \t{space}{underscore}")?;
        cwriteln!(f, "    \t<g!>{}</>", self.fix)
    }
}

pub trait Expression<T> {
    fn eval(&mut self) -> Result<T, PartialEvalError>;
}
