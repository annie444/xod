use crate::parsers::{Span, general::lines};
use color_print::cformat;
use rustyline::{
    Context, Helper, Hinter,
    completion::{Candidate, Completer},
    error::ReadlineError,
    highlight::{CmdKind, Highlighter},
    hint::HistoryHinter,
    history::SearchDirection,
    validate::{ValidationContext, ValidationResult, Validator},
};
use std::{borrow::Cow, cell::Cell};

#[derive(Hinter)]
pub struct XodHelper {
    #[rustyline(Hinter)]
    hinter: HistoryHinter,
    bracket: Cell<Option<(u8, usize)>>, // (bracket, position)
}

impl Default for XodHelper {
    fn default() -> Self {
        Self {
            hinter: HistoryHinter::new(),
            bracket: Cell::new(None),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XodCandidate {
    pub display: String,
    pub replacement: String,
}

impl Candidate for XodCandidate {
    fn display(&self) -> &str {
        &self.display
    }

    fn replacement(&self) -> &str {
        &self.replacement
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Delimiters {
    Paren(usize),
    Bracket(usize),
    Brace(usize),
}

fn balanced(line: &str) -> Vec<Delimiters> {
    let mut paren_stack: u16 = 0;
    let mut last_open_paren: Option<usize> = None;
    let mut bracket_stack: u16 = 0;
    let mut last_open_bracket: Option<usize> = None;
    let mut brace_stack: u16 = 0;
    let mut last_open_brace: Option<usize> = None;
    let mut order = Vec::new();
    for (i, c) in line.chars().enumerate() {
        if c == '(' {
            paren_stack += 1;
            last_open_paren = Some(i);
        } else if c == ')' {
            paren_stack -= 1;
        }
        if c == '[' {
            bracket_stack += 1;
            last_open_bracket = Some(i);
        } else if c == ']' {
            bracket_stack -= 1;
        }
        if c == '{' {
            brace_stack += 1;
            last_open_brace = Some(i);
        } else if c == '}' {
            brace_stack -= 1;
        }
    }
    if paren_stack != 0
        && let Some(last_paren) = last_open_paren
    {
        order.push(Delimiters::Paren(last_paren));
    }
    if bracket_stack != 0
        && let Some(last_bracket) = last_open_bracket
    {
        order.push(Delimiters::Bracket(last_bracket));
    }
    if brace_stack != 0
        && let Some(last_brace) = last_open_brace
    {
        order.push(Delimiters::Brace(last_brace));
    }
    order.sort();
    order
}

impl Completer for XodHelper {
    type Candidate = XodCandidate;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context,
    ) -> Result<(usize, Vec<Self::Candidate>), ReadlineError> {
        // Here you would implement your completion logic
        // For demonstration, we return a static list of candidates
        let mut candidates = Vec::new();
        let start = if ctx.history_index() == ctx.history().len() {
            ctx.history_index().saturating_sub(1)
        } else {
            ctx.history_index()
        };
        if let Some(search) = ctx
            .history()
            .starts_with(line, start, SearchDirection::Reverse)
            .unwrap_or(None)
        {
            let display = search.entry.to_string();
            let replacement = display[search.pos..].to_string();
            candidates.push(XodCandidate {
                display: display.clone(),
                replacement: replacement.clone(),
            });
        }
        let order = balanced(line);
        for d in order {
            let display = match d {
                Delimiters::Paren(_) => ")".to_string(),
                Delimiters::Bracket(_) => "]".to_string(),
                Delimiters::Brace(_) => "}".to_string(),
            };
            let replacement = display.clone();
            candidates.push(XodCandidate {
                display,
                replacement,
            });
        }
        Ok((pos, candidates))
    }
}

impl Highlighter for XodHelper {
    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        if line.len() <= 1 {
            return Cow::Borrowed(line);
        }
        // highlight matching brace/bracket/parenthesis if it exists
        if let Some((bracket, pos)) = self.bracket.get()
            && let Some((matching, idx)) = find_matching_bracket(line, pos, bracket)
        {
            let mut copy = line.to_owned();
            copy.replace_range(idx..=idx, &cformat!("<s><g>{}</></>", matching as char));
            return Cow::Owned(copy);
        }
        Cow::Borrowed(line)
    }

    fn highlight_char(&self, line: &str, pos: usize, kind: CmdKind) -> bool {
        if kind == CmdKind::ForcedRefresh {
            self.bracket.set(None);
            return false;
        }
        // will highlight matching brace/bracket/parenthesis if it exists
        self.bracket.set(check_bracket(line, pos));
        self.bracket.get().is_some()
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        // Highlight the hint with a specific style
        Cow::Owned(cformat!("<dim><i>{hint}</></>"))
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            // Highlight the prompt if it's the default prompt
            Cow::Owned(cformat!("<s><c>{prompt}</></>"))
        } else {
            // If not the default prompt, return it as is
            Cow::Owned(cformat!("<s><m>{prompt}</></>"))
        }
    }
}

fn find_matching_bracket(line: &str, pos: usize, bracket: u8) -> Option<(u8, usize)> {
    let matching = matching_bracket(bracket);
    let mut idx;
    let mut unmatched = 1;
    if is_open_bracket(bracket) {
        // forward search
        idx = pos + 1;
        let bytes = &line.as_bytes()[idx..];
        for b in bytes {
            if *b == matching {
                unmatched -= 1;
                if unmatched == 0 {
                    debug_assert_eq!(matching, line.as_bytes()[idx]);
                    return Some((matching, idx));
                }
            } else if *b == bracket {
                unmatched += 1;
            }
            idx += 1;
        }
        debug_assert_eq!(idx, line.len());
    } else {
        // backward search
        idx = pos;
        let bytes = &line.as_bytes()[..idx];
        for b in bytes.iter().rev() {
            if *b == matching {
                unmatched -= 1;
                if unmatched == 0 {
                    debug_assert_eq!(matching, line.as_bytes()[idx - 1]);
                    return Some((matching, idx - 1));
                }
            } else if *b == bracket {
                unmatched += 1;
            }
            idx -= 1;
        }
        debug_assert_eq!(idx, 0);
    }
    None
}

// check under or before the cursor
fn check_bracket(line: &str, pos: usize) -> Option<(u8, usize)> {
    if line.is_empty() {
        return None;
    }
    let mut pos = pos;
    if pos >= line.len() {
        pos = line.len() - 1; // before cursor
        let b = line.as_bytes()[pos]; // previous byte
        if is_close_bracket(b) {
            Some((b, pos))
        } else {
            None
        }
    } else {
        let mut under_cursor = true;
        loop {
            let b = line.as_bytes()[pos];
            if is_close_bracket(b) {
                return if pos == 0 { None } else { Some((b, pos)) };
            } else if is_open_bracket(b) {
                return if pos + 1 == line.len() {
                    None
                } else {
                    Some((b, pos))
                };
            } else if under_cursor && pos > 0 {
                under_cursor = false;
                pos -= 1; // or before cursor
            } else {
                return None;
            }
        }
    }
}

const fn matching_bracket(bracket: u8) -> u8 {
    match bracket {
        b'{' => b'}',
        b'}' => b'{',
        b'[' => b']',
        b']' => b'[',
        b'(' => b')',
        b')' => b'(',
        b => b,
    }
}
const fn is_open_bracket(bracket: u8) -> bool {
    matches!(bracket, b'{' | b'[' | b'(')
}
const fn is_close_bracket(bracket: u8) -> bool {
    matches!(bracket, b'}' | b']' | b')')
}

const FUNCTIONS: [&str; 12] = [
    "exit", "quit", "hex", "bin", "dec", "oct", "bool", "clear", "history", "hist", "help", "range",
];

impl Validator for XodHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> Result<ValidationResult, ReadlineError> {
        if ctx.input().is_empty() {
            return Ok(ValidationResult::Valid(None));
        }
        let brackets = validate_brackets(ctx.input());
        if matches!(brackets, ValidationResult::Incomplete)
            || matches!(brackets, ValidationResult::Invalid(_))
        {
            return Ok(brackets);
        }
        let func = validate_function(ctx.input());
        if matches!(func, ValidationResult::Invalid(_)) {
            return Ok(func);
        }
        Ok(validate_parse(ctx.input()))
    }
}

fn validate_function(src: &str) -> ValidationResult {
    for func in FUNCTIONS {
        if src.contains(func) {
            let f = src.find(func).unwrap();
            if !src[f + func.len()..].contains(['(']) && !src[f + func.len()..].contains([')']) {
                return ValidationResult::Invalid(Some(format!(
                    "\nFunction `{func}` requires parentheses: `{func}()`",
                )));
            }
        }
    }
    ValidationResult::Valid(None)
}

fn validate_parse(src: &str) -> ValidationResult {
    let input_str = format!("{src}\n");
    let input = Span::new(&input_str);
    match lines(input) {
        Ok((_, _)) => ValidationResult::Valid(None),
        Err(e) => match e {
            nom::Err::Incomplete(_) => ValidationResult::Incomplete,
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                let mut error = String::new();
                error.push_str(&cformat!(
                    "\n<s><r!>error</>: {}</>\n",
                    e.code.description()
                ));
                let start = e.input.naive_get_utf8_column();
                let end = e.input.fragment().len();
                let line = e.input.location_line();
                error.push_str(&cformat!(
                    "<s><b!> --></> line {line}, cols {start}-{}</>\n",
                    start + end,
                ));
                let line = cformat!("<s><b!>|</></>");
                let body: String = e
                    .input
                    .fragment()
                    .split('\n')
                    .enumerate()
                    .map(|(i, s)| cformat!("\n{line} <b!>{i:0>2}</>\t{s}"))
                    .collect();
                error.push_str(&line);
                error.push_str(&body);
                error.push_str(&format!("\n{line}\n"));
                ValidationResult::Invalid(Some(error))
            }
        },
    }
}

fn validate_brackets(input: &str) -> ValidationResult {
    let mut stack = vec![];
    for c in input.chars() {
        match c {
            '(' | '[' | '{' => stack.push(c),
            ')' | ']' | '}' => match (stack.pop(), c) {
                (Some('('), ')') | (Some('['), ']') | (Some('{'), '}') => {}
                (Some(wanted), _) => {
                    return ValidationResult::Invalid(Some(format!(
                        "\nMismatched brackets: {wanted:?} is not properly closed"
                    )));
                }
                (None, c) => {
                    return ValidationResult::Invalid(Some(format!(
                        "\nMismatched brackets: {c:?} is unpaired"
                    )));
                }
            },
            _ => {}
        }
    }
    if stack.is_empty() {
        ValidationResult::Valid(None)
    } else {
        ValidationResult::Incomplete
    }
}

impl Helper for XodHelper {}
