pub mod helper;

use crate::parsers::{
    EvalError, ExprError, Expression, Span, exprs::NumOrListNoOp, general::lines,
};
use crate::utils::print_num;
use nom::Parser;
use rustyline::{
    Behavior, ColorMode, CompletionType, EditMode, Editor,
    config::Config,
    error::ReadlineError,
    history::{FileHistory, History},
};
use shellexpand::tilde;
use std::fs::File;
use std::path::Path;

use self::helper::XodHelper;

pub fn run() {
    println!("REPL is not implemented yet.");
    // Here you would typically set up the REPL loop, read user input,
    // parse commands, and execute them.
    // For example:
    // loop {
    //     let input = get_user_input();
    //     match parse_command(input) {
    //         Ok(command) => execute_command(command),
    //         Err(e) => println!("Error: {}", e),
    //     }
    // }

    let file = tilde("~/.local/cache/xod/history").to_string();
    let history_file = Path::new(&file);
    if !history_file.exists() {
        if let Some(parent) = history_file.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create history directory");
        }
        File::create(&history_file).expect("Failed to create history file");
        FileHistory::new()
            .save(&history_file)
            .expect("Failed to create history file");
    }
    let mut history = FileHistory::new();
    history
        .load(&history_file)
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
        .edit_mode(EditMode::Vi)
        .auto_add_history(true)
        .color_mode(ColorMode::Enabled)
        .behavior(Behavior::PreferTerm)
        .tab_stop(8)
        .check_cursor_position(true)
        .indent_size(4)
        .bracketed_paste(true)
        .enable_signals(true)
        .build();
    let helper = XodHelper::new();
    let mut rl: Editor<XodHelper, FileHistory> =
        Editor::with_history(config, history).expect("Failed to create editor");
    rl.set_helper(Some(helper));
    let mut ctx: Option<String> = None;
    loop {
        let readline = match ctx {
            Some(ref ctx_ref) => rl.readline_with_initial(">> ", (&ctx_ref, "")),
            None => rl.readline(">> "),
        };
        ctx = None;
        match readline {
            Ok(mut line) => {
                rl.add_history_entry(line.as_str()).unwrap();
                // Here you would parse and execute the command
                line.push_str("\n");
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
                for parsed_line in &mut parsed_lines {
                    match parsed_line.eval() {
                        Ok(result) => match result {
                            NumOrListNoOp::Num(n) => print_num("Result:", n),
                            NumOrListNoOp::List(l) => println!("List: {:?}", l),
                            NumOrListNoOp::NoOp => (),
                        },
                        Err(e) => match e {
                            ExprError::Quit => {
                                println!("Exiting REPL.");
                                return;
                            }
                            ExprError::Partial(p) => {
                                eprintln!("{}", EvalError::from((p, body)))
                            }
                            ExprError::Print(s) => println!("{}", s),
                        },
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Ctrl-C pressed, exiting REPL.");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("Ctrl-D pressed, exiting REPL.");
                break;
            }
            Err(err) => {
                println!("Error reading line: {:?}", err);
                break;
            }
        }
    }
}
