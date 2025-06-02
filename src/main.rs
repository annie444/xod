use clap::{CommandFactory, Parser, error::ErrorKind};
use xod::{bitops::BitOps, cli_parser::NumberParser, repl::run, utils::print_num};

/// Lightweight binary number calculator.
///
/// Useful for converting numbers between binary, hexadecimal, octal, decimal, and single bit
/// representations. Supply an operator to see how it modified the number, and include another
/// value to do some quick and easy bitwise arithmatic.
#[derive(Parser, Debug)]
pub struct HexOctBin {
    /// The main numerical value in either hexadecimal (0x), binary (0b), octal (0o), or decimal
    /// (default) form.
    #[clap(value_parser = NumberParser::new())]
    pub number: Option<usize>,

    /// A bitwise operator: AND (&), OR (|), NOT (! or ~), XOR (^), LEFT SHIFT (<<), RIGHT SHIFT (>>)
    pub operation: Option<BitOps>,

    /// The other numerical value to apply to the main value using the bitwise operator.
    #[clap(value_parser = NumberParser::new())]
    pub other: Option<usize>,
}

fn main() {
    let args = HexOctBin::parse();
    match args.number {
        Some(_) => {
            print_nums(args);
        }
        None => {
            run();
        }
    }
}

fn print_nums(args: HexOctBin) {
    let number = args.number.unwrap();
    print_num("Input Number:", number);

    if args.operation.is_some() && args.other.is_some() {
        let op = args.operation.unwrap();
        let other = args.other.unwrap();

        let result = match op {
            BitOps::Xor => number ^ other,
            BitOps::LeftShift => number << other,
            BitOps::Not => !number,
            BitOps::Or => number | other,
            BitOps::RightShift => number >> other,
            BitOps::And => number & other,
        };
        print_num("Other Number:", other);
        print_num("Resulting Value:", result);
    } else if args.operation.is_some() {
        let op = args.operation.unwrap();
        let result = match op {
            BitOps::Not => !number,
            _ => HexOctBin::command()
                .error(
                    ErrorKind::InvalidValue,
                    format!(
                        r#"Cannot use operator {} without another number.

Example:
    {} {} 0xff
                "#,
                        op, number, op
                    ),
                )
                .exit(),
        };
        print_num("Resulting Value:", result);
    }
}
