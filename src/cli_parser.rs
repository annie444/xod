use crate::bitops::BitOps;
use clap::{
    Arg, Command,
    builder::{StringValueParser, TypedValueParser},
    error::{ContextKind, ContextValue, ErrorKind},
};
use std::{
    ffi::OsStr,
    num::{IntErrorKind, ParseIntError},
};

#[derive(Clone, Debug)]
pub struct BitOpsParser;

impl Default for BitOpsParser {
    fn default() -> Self {
        Self::new()
    }
}

impl BitOpsParser {
    pub fn new() -> Self {
        Self
    }
}

impl TypedValueParser for BitOpsParser {
    type Value = BitOps;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let str_parser = StringValueParser::new();
        let val = str_parser.parse_ref(cmd, arg, value)?;
        match val.to_ascii_lowercase().as_str() {
            "&" | "and" => Ok(BitOps::And),
            "|" | "or" => Ok(BitOps::Or),
            "^" | "xor" => Ok(BitOps::Xor),
            "<<" | "left" => Ok(BitOps::LeftShift),
            ">>" | "right" => Ok(BitOps::RightShift),
            "!" | "~" | "not" => Ok(BitOps::Not),
            "+" | "plus" | "add" => Ok(BitOps::Add),
            "-" | "minus" | "sub" | "subtract" => Ok(BitOps::Subtract),
            "*" | "x" | "times" | "mul" | "multiply" => Ok(BitOps::Multiply),
            "/" | "div" | "divide" => Ok(BitOps::Divide),
            "%" | "mod" | "modulo" => Ok(BitOps::Modulo),
            "**" | "pow" | "power" | "expo" | "exponent" => Ok(BitOps::Expo),
            _ => {
                let mut error = clap::Error::raw(
                    ErrorKind::InvalidValue,
                    format!(
                        r#"{} is not a valid binary operator.
Valid operators are:

    Bitwise AND : &
    Bitwise OR  : |
    Bitwise XOR : ^
    Bitwise NOT : ! or ~
    Left shift  : <<
    Right shift : >>
                "#,
                        val
                    ),
                );
                error = error.with_cmd(cmd);
                let _ = error.insert(
                    ContextKind::InvalidValue,
                    ContextValue::String("number".to_string()),
                );
                Err(error)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct NumberParser;

impl Default for NumberParser {
    fn default() -> Self {
        Self::new()
    }
}

impl NumberParser {
    pub fn new() -> Self {
        Self
    }
}

impl TypedValueParser for NumberParser {
    type Value = usize;

    // Required method
    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let err = |e: ParseIntError| {
            let kind = match e.kind() {
                IntErrorKind::Empty => "No number passed",
                IntErrorKind::InvalidDigit => "Invalid digit",
                IntErrorKind::PosOverflow => "Digit overflow",
                IntErrorKind::NegOverflow => "Digit underflow",
                IntErrorKind::Zero => "Zero error",
                _ => "Unknown error occoured when parsing the numbers...",
            };
            let mut error = clap::Error::raw(
                ErrorKind::InvalidValue,
                format!("Unable to parse number, got error: {}", kind),
            );
            error = error.with_cmd(cmd);
            let _ = error.insert(
                ContextKind::InvalidValue,
                ContextValue::Strings(vec!["number".to_string(), "other".to_string()]),
            );
            error
        };
        let str_parser = StringValueParser::new();
        let val = str_parser.parse_ref(cmd, arg, value)?;
        if val.starts_with("0x") {
            match usize::from_str_radix(val.strip_prefix("0x").unwrap(), 16) {
                Ok(num) => Ok(num),
                Err(e) => Err(err(e)),
            }
        } else if val.starts_with("0b") {
            match usize::from_str_radix(val.strip_prefix("0b").unwrap(), 2) {
                Ok(num) => Ok(num),
                Err(e) => Err(err(e)),
            }
        } else if val.starts_with("0o") {
            match usize::from_str_radix(val.strip_prefix("0o").unwrap(), 8) {
                Ok(num) => Ok(num),
                Err(e) => Err(err(e)),
            }
        } else {
            match val.parse::<usize>() {
                Ok(num) => Ok(num),
                Err(e) => Err(err(e)),
            }
        }
    }
}
