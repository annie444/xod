pub mod event;
pub mod help;
pub mod helper;

use crate::parsers::PartialEvalError;
use crate::parsers::ast::Line;
use crate::parsers::{
    EvalError, ExprError, Expression, Span, exprs::NumOrListNoOp, general::lines,
};
use crate::utils::print_num;
use color_print::{cformat, cprintln};
use rustyline::{
    Behavior, Cmd, ColorMode, CompletionType, Editor, Event, EventHandler, KeyEvent,
    config::Config,
    error::ReadlineError,
    history::{FileHistory, History},
};
use shellexpand::tilde;
use std::collections::VecDeque;
use std::fs::File;
use std::path::Path;

use self::{
    event::{XodCompleteHintHandler, XodTabEventHandler},
    help::print_help,
    helper::XodHelper,
};

pub fn run() {
    println!("REPL is not implemented yet.");
    cprintln!(
        r#"
<s><m>Welcome to the Xod REPL!</></>

    This REPL allows you to evaluate bitwise expressions interactively. 
    You can enter any valid Xod expression, and it will be evaluated immediately.

    Type <s><g>help()</></> for a list of commands and a breakdown of the syntax.
    Type <s><g>exit()</></> to exit the REPL.
"#
    );

    let file = tilde("~/.local/cache/xod/history").to_string();
    let history_file = Path::new(&file);
    if !history_file.exists() {
        if let Some(parent) = history_file.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create history directory");
        }
        File::create(history_file).expect("Failed to create history file");
        FileHistory::new()
            .save(history_file)
            .expect("Failed to create history file");
    }
    let mut history = FileHistory::new();
    history
        .load(history_file)
        .expect("Failed to load history file");

    let config = Config::builder()
        .max_history_size(1000)
        .expect("Failed to set max history size")
        .history_ignore_dups(true)
        .expect("Failed to ignore duplicates")
        .history_ignore_space(true)
        .completion_prompt_limit(50)
        .completion_type(CompletionType::Fuzzy)
        .completion_show_all_if_ambiguous(false)
        .auto_add_history(true)
        .color_mode(ColorMode::Enabled)
        .behavior(Behavior::PreferTerm)
        .tab_stop(8)
        .check_cursor_position(true)
        .indent_size(4)
        .bracketed_paste(true)
        .enable_signals(true)
        .build();

    let helper = XodHelper::default();
    let complete_handler = Box::new(XodCompleteHintHandler);

    let mut rl: Editor<XodHelper, FileHistory> =
        Editor::with_history(config, history).expect("Failed to create editor");
    rl.set_helper(Some(helper));
    rl.bind_sequence(
        KeyEvent::ctrl('E'),
        EventHandler::Conditional(complete_handler.clone()),
    );
    rl.bind_sequence(
        KeyEvent::alt('f'),
        EventHandler::Conditional(complete_handler),
    );
    rl.bind_sequence(
        Event::KeySeq(vec![KeyEvent::ctrl('X'), KeyEvent::ctrl('E')]),
        EventHandler::Simple(Cmd::Suspend),
    );
    rl.bind_sequence(
        KeyEvent::from('\t'),
        EventHandler::Conditional(Box::new(XodTabEventHandler)),
    );

    let mut ctx: Option<String> = None;
    loop {
        let readline = match ctx {
            Some(ref ctx_ref) => rl.readline_with_initial(">> ", (ctx_ref, "")),
            None => rl.readline(">> "),
        };
        ctx = None;
        match readline {
            Ok(mut line) => {
                rl.add_history_entry(line.as_str()).unwrap();
                // Here you would parse and execute the command
                line.push('\n');
                let body = Span::new(&line);
                let mut parsed_lines = match lines(body) {
                    Ok((_, l)) => l,
                    Err(e) => {
                        match e {
                            nom::Err::Error(e) => {
                                let e_str = e.input.fragment().to_string();
                                let mut chars = e_str.chars();
                                chars.next_back();
                                ctx = Some(chars.as_str().to_owned());
                                continue;
                            }
                            _ => {
                                continue;
                            }
                        };
                    }
                };
                match parse_lines(&mut parsed_lines) {
                    XodCmd::Help => {
                        print_help();
                        continue;
                    }
                    XodCmd::History => {
                        print_history(&rl);
                        continue;
                    }
                    XodCmd::Clear => {
                        let _ = rl.clear_screen();
                        continue;
                    }
                    XodCmd::Quit => {
                        println!("Exiting REPL.");
                        rl.history_mut()
                            .save(history_file)
                            .expect("Failed to save history file");
                        break;
                    }
                    XodCmd::NoOp => continue,
                    XodCmd::Error(e) => {
                        eprintln!("{}", EvalError::from((e, body)));
                        continue;
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Ctrl-C pressed, exiting REPL.");
                rl.history_mut()
                    .save(history_file)
                    .expect("Failed to save history file");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("Ctrl-D pressed, exiting REPL.");
                rl.history_mut()
                    .save(history_file)
                    .expect("Failed to save history file");
                break;
            }
            Err(err) => {
                println!("Error reading line: {err:?}");
                rl.history_mut()
                    .save(history_file)
                    .expect("Failed to save history file");
                break;
            }
        }
    }
}

pub enum XodCmd<'a> {
    Help,
    History,
    Clear,
    Quit,
    NoOp,
    Error(PartialEvalError<'a>),
}

fn print_history(rl: &Editor<XodHelper, FileHistory>) {
    let len = rl.history().len();
    for (i, entry) in rl.history().iter().enumerate() {
        let i = len - i;
        let mut entry = entry
            .split('\n')
            .map(|s| format!("      {s}\n"))
            .collect::<String>();
        entry.replace_range(0..=4, &cformat!("<s><b>{i: >3}</></> :"));
        println!("{entry}");
    }
}

fn parse_lines<'a>(parsed_lines: &'a mut VecDeque<Line>) -> XodCmd<'a> {
    for parsed_line in parsed_lines.iter_mut() {
        match parsed_line.eval() {
            Ok(result) => match result {
                NumOrListNoOp::Num(n) => print_num("", n),
                NumOrListNoOp::List(l) => println!("{l:?}"),
                NumOrListNoOp::NoOp => {}
            },
            Err(e) => match e {
                ExprError::Quit => return XodCmd::Quit,
                ExprError::Help => return XodCmd::Help,
                ExprError::History => return XodCmd::History,
                ExprError::Clear => return XodCmd::Clear,
                ExprError::Partial(p) => return XodCmd::Error(p),
            },
        }
    }
    XodCmd::NoOp
}
