use crate::cli_parser::BitOpsParser;
use clap::builder::ValueParserFactory;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BitOps {
    And,
    Or,
    Xor,
    LeftShift,
    RightShift,
    Not,
}

impl fmt::Display for BitOps {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::And => write!(f, "&"),
            Self::Or => write!(f, "|"),
            Self::Xor => write!(f, "^"),
            Self::LeftShift => write!(f, "<<"),
            Self::RightShift => write!(f, ">>"),
            Self::Not => write!(f, "~ or !"),
        }
    }
}

impl ValueParserFactory for BitOps {
    type Parser = BitOpsParser;

    fn value_parser() -> Self::Parser {
        BitOpsParser::new()
    }
}
