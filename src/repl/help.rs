use color_print::cprintln;
use prettytable::{
    Attr, Cell, Row, Table, color,
    format::{Alignment, FormatBuilder, LinePosition, LineSeparator, TableFormat},
};

// Using cardinal directions
// E.g. Character ╯:
//   N
// W ╯ E
//   S
// ╯ = North-West = NW
pub(crate) const SE: char = '╭';
pub(crate) const NE: char = '╰';
const SW: char = '╮';
pub(crate) const NS: char = '│';
const NW: char = '╯';
const SEW: char = '┬';
const NSE: char = '├';
const NSW: char = '┤';
const NEW: char = '┴';
const NSEW: char = '┼';
pub(crate) const EW: char = '─';

pub fn print_help() {
    // Set up the table format
    let table_format = FormatBuilder::new()
        .padding(2, 2)
        .column_separator(NS)
        .borders(NS)
        .separator(LinePosition::Top, LineSeparator::new(EW, SEW, SE, SW))
        .separator(LinePosition::Bottom, LineSeparator::new(EW, NEW, NE, NW))
        .separators(
            &[LinePosition::Title, LinePosition::Intern],
            LineSeparator::new(EW, NSEW, NSE, NSW),
        )
        .indent(4)
        .build();

    let commands = commands_table(table_format);
    let bit_operators = bitwise_operators_table(table_format);
    let cmp_operators = comparison_operators_table(table_format);
    let list_methods = list_methods_table(table_format);
    let loops = loops_table(table_format);

    cprintln!(
        r#"
<s><m>Welcome to the Xod REPL!</></>

    This REPL allows you to evaluate bitwise expressions interactively. You can enter any valid Xod expression, and it will be evaluated immediately. Below is a breakdown of the domain specific language (DSL) and commands available in this REPL: 

<s><y!>Commands:</></>
"#
    );
    commands.printstd();
    cprintln!(
        r#"
<s><y!>Bitwise operators:</></>
"#
    );
    bit_operators.printstd();
    cprintln!(
        r#"
<s><y!>Boolean operators:</></>
"#
    );
    cmp_operators.printstd();
    cprintln!(
        r#"
<s><y!>List methods:</></>
"#
    );
    list_methods.printstd();
    cprintln!(
        r#"
<s><y!>Block statements:</></>
"#
    );
    loops.printstd();
    cprintln!(
        r#"
<s><r!>Note:</></>

    There is not an agreed upon standard for the order of operations in bitwise expressions. To avoid ambiguity, it is required to use parentheses to group chained expressions. For example, instead of writing `a & b | c`, you should write `(a & b) | c` or `a & (b | c)` to clarify the order of operations.

"#
    );
}

fn loops_table(table_format: TableFormat) -> Table {
    let mut loops = Table::new();
    loops.set_format(table_format);
    loops.set_titles(Row::new(vec![
        Cell::new_align("Block", Alignment::LEFT)
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::BRIGHT_CYAN)),
        Cell::new_align("Arguments", Alignment::LEFT)
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::BRIGHT_CYAN)),
        Cell::new_align("Description", Alignment::LEFT)
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::BRIGHT_CYAN)),
    ]));
    loops.extend(vec![
        Row::new(vec![
            Cell::new_align(
                "for(<var> in <iterable>) { ... }",
                Alignment::CENTER,
            ).with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("A variable and an iterable. The iterable can be a range or a list.\nThe value of the iterator is assigned to the variable.", Alignment::LEFT),
            Cell::new_align("Iterate over a range or iterable.", Alignment::LEFT),
        ]),
        Row::new(vec![
            Cell::new_align(
                "while(<condition>) { ... }",
                Alignment::CENTER,
            ).with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("A conditional statement. (See 'Boolean operators')", Alignment::LEFT),
            Cell::new_align("Repeat while the condition is true.", Alignment::LEFT),
        ]),
        Row::new(vec![
            Cell::new_align(
                "if(<condition>) { ... }",
                Alignment::CENTER,
            ).with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("A conditional statement. (See 'Boolean operators')", Alignment::LEFT),
            Cell::new_align(
                "Execute the block if the condition is true.",
                Alignment::LEFT,
            ),
        ]),
    ]);
    loops
}

fn list_methods_table(table_format: TableFormat) -> Table {
    let mut methods = Table::new();
    methods.set_format(table_format);
    methods.set_titles(Row::new(vec![
        Cell::new_align("Method", Alignment::LEFT)
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::BRIGHT_CYAN)),
        Cell::new_align("Arguments", Alignment::LEFT)
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::BRIGHT_CYAN)),
        Cell::new_align("Description", Alignment::LEFT)
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::BRIGHT_CYAN)),
    ]));
    methods.extend(vec![
        Row::new(vec![
            Cell::new_align("append(<value>)", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("A variable or number", Alignment::LEFT),
            Cell::new_align("Append a value to the back of the list.", Alignment::LEFT),
        ]),
        Row::new(vec![
            Cell::new_align("prepend(<value>)", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("A variable or number", Alignment::LEFT),
            Cell::new_align("Prepend a value to the front of the list.", Alignment::LEFT),
        ]),
        Row::new(vec![
            Cell::new_align("back()", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("", Alignment::LEFT),
            Cell::new_align(
                "Remove and return the last value from the list.",
                Alignment::LEFT,
            ),
        ]),
        Row::new(vec![
            Cell::new_align("front()", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("", Alignment::LEFT),
            Cell::new_align(
                "Remove and return the first value from the list.",
                Alignment::LEFT,
            ),
        ]),
        Row::new(vec![
            Cell::new_align("index(<index>)", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("A variable or number", Alignment::LEFT),
            Cell::new_align(
                "Get the element at the specified index (0 indexed).",
                Alignment::LEFT,
            ),
        ]),
    ]);
    methods
}

fn comparison_operators_table(table_format: TableFormat) -> Table {
    let mut operators = Table::new();
    operators.set_format(table_format);
    operators.set_titles(Row::new(vec![
        Cell::new_align("Operator", Alignment::LEFT)
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::BRIGHT_CYAN)),
        Cell::new_align("Description", Alignment::LEFT)
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::BRIGHT_CYAN)),
    ]));
    operators.extend(vec![
        Row::new(vec![
            Cell::new_align("==", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("Equality operator.", Alignment::LEFT),
        ]),
        Row::new(vec![
            Cell::new_align("!=", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("Inequality operator.", Alignment::LEFT),
        ]),
        Row::new(vec![
            Cell::new_align("<", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("Less than operator.", Alignment::LEFT),
        ]),
        Row::new(vec![
            Cell::new_align(">", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("Greater than operator.", Alignment::LEFT),
        ]),
        Row::new(vec![
            Cell::new_align("<=", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("Less than or equal to operator.", Alignment::LEFT),
        ]),
        Row::new(vec![
            Cell::new_align(">=", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("Greater than or equal to operator.", Alignment::LEFT),
        ]),
    ]);
    operators
}

fn bitwise_operators_table(table_format: TableFormat) -> Table {
    let mut operators = Table::new();
    operators.set_format(table_format);
    operators.set_titles(Row::new(vec![
        Cell::new_align("Operator", Alignment::LEFT)
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::BRIGHT_CYAN)),
        Cell::new_align("Description", Alignment::LEFT)
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::BRIGHT_CYAN)),
    ]));
    operators.extend(vec![
        Row::new(vec![
            Cell::new_align("&", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("Bitwise AND operator.", Alignment::LEFT),
        ]),
        Row::new(vec![
            Cell::new_align("|", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("Bitwise OR operator.", Alignment::LEFT),
        ]),
        Row::new(vec![
            Cell::new_align("^", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("Bitwise XOR operator.", Alignment::LEFT),
        ]),
        Row::new(vec![
            Cell::new_align("! or ~", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("Bitwise NOT operator.", Alignment::LEFT),
        ]),
        Row::new(vec![
            Cell::new_align("<<", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("Bitwise left shift operator.", Alignment::LEFT),
        ]),
        Row::new(vec![
            Cell::new_align(">>", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("Bitwise right shift operator.", Alignment::LEFT),
        ]),
    ]);
    operators
}

fn commands_table(table_format: TableFormat) -> Table {
    let mut commands = Table::new();
    commands.set_format(table_format);
    commands.set_titles(Row::new(vec![
        Cell::new_align("Command", Alignment::LEFT)
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::BRIGHT_CYAN)),
        Cell::new_align("Args", Alignment::LEFT)
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::BRIGHT_CYAN)),
        Cell::new_align("Description", Alignment::LEFT)
            .with_style(Attr::Bold)
            .with_style(Attr::ForegroundColor(color::BRIGHT_CYAN)),
    ]));
    commands.extend(vec![
        Row::new(vec![
            Cell::new_align("help()", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("", Alignment::LEFT),
            Cell::new_align("Show this help message.", Alignment::LEFT),
        ]),
        Row::new(vec![
            Cell::new_align("quit()", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("", Alignment::LEFT),
            Cell::new_align("Exit the REPL.", Alignment::LEFT),
        ]),
        Row::new(vec![
            Cell::new_align("clear()", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("", Alignment::LEFT),
            Cell::new_align("Clear the screen.", Alignment::LEFT),
        ]),
        Row::new(vec![
            Cell::new_align("history()", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("", Alignment::LEFT),
            Cell::new_align("Show command history.", Alignment::LEFT),
        ]),
        Row::new(vec![
            Cell::new_align("bool(<stmt>)", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("A boolean comparison or a variable", Alignment::LEFT),
            Cell::new_align(
                "Returns the input as a boolean value (true = 1, false = 0).",
                Alignment::LEFT,
            ),
        ]),
        Row::new(vec![
            Cell::new_align("range(<value>, <value>)", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align(
                "Two values of either a variable or a number",
                Alignment::LEFT,
            ),
            Cell::new_align(
                "Returns a non-inclusive iterator over the range.",
                Alignment::LEFT,
            ),
        ]),
        Row::new(vec![
            Cell::new_align("hex(<value>)", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("A variable or a number", Alignment::LEFT),
            Cell::new_align("Prints the input as a hexadecimal number.", Alignment::LEFT),
        ]),
        Row::new(vec![
            Cell::new_align("oct(<value>)", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("A variable or a number", Alignment::LEFT),
            Cell::new_align("Prints the input as a octal number.", Alignment::LEFT),
        ]),
        Row::new(vec![
            Cell::new_align("bin(<value>)", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("A variable or a number", Alignment::LEFT),
            Cell::new_align("Prints the input as a binary number.", Alignment::LEFT),
        ]),
        Row::new(vec![
            Cell::new_align("dec(<value>)", Alignment::CENTER)
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::BRIGHT_GREEN)),
            Cell::new_align("A variable or a number", Alignment::LEFT),
            Cell::new_align("Prints the input as a decimal number.", Alignment::LEFT),
        ]),
    ]);
    commands
}
