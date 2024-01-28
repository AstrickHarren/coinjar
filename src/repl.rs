mod date;
mod split;
mod util;

use std::fmt::Display;

use anyhow::{Context, Result};
use chrono::{Local, NaiveDate};
use colored::Colorize;
use itertools::Itertools;
use pest::Parser;
use rustyline::{config::Configurer, error::ReadlineError};

use crate::journal::{
    parser::{IdentParser, Rule},
    register::QueryType,
    Journal, Txn,
};

use self::date::DateArg;

struct ReplState {
    date: NaiveDate,
    file: String,
    new_txns: Vec<Txn>,
}

#[derive(Debug, clap::Parser)]
struct Args {
    file: String,
}

pub(crate) fn repl() {
    let history_path = "/tmp/coinjar.history";

    let (args, mut journal) = parse_args().unwrap_or_else(|e| exit_gracefully(e));
    let mut rl = rustyline::DefaultEditor::new().unwrap_or_else(|e| exit_gracefully(e));
    rl.load_history(history_path).ok();
    rl.set_auto_add_history(true);
    let mut state = ReplState {
        date: Local::now().date_naive(),
        file: args.file.clone(),
        new_txns: Vec::new(),
    };

    loop {
        let ret: Result<()> = try {
            let input = rl.readline("coinjar> ");
            let input = match input {
                Err(ReadlineError::Interrupted) => continue,
                Err(ReadlineError::Eof) => {
                    rl.save_history(history_path)
                        .unwrap_or_else(|e| exit_gracefully(e));
                    return;
                }
                input => input?,
            };

            interact(&input, &mut journal, &mut state)?;
        };

        ret.with_context(|| format!("{}", "error".red().bold()))
            .unwrap_or_else(|e| eprintln!("{:#}", e));
    }
}

fn interact(input: &str, journal: &mut Journal, state: &mut ReplState) -> Result<()> {
    let pair = IdentParser::parse(Rule::cmd, input)
        .with_context(|| "Failed to parse cmd".to_string())?
        .next()
        .unwrap();

    match pair.as_rule() {
        Rule::date_cmd => {
            let date_arg = pair.into_inner().next();
            if let Some(d) = date_arg
                .map(|d| d.as_str().parse::<DateArg>())
                .transpose()?
            {
                d.apply(&mut state.date)
            }
            println!("{}", state.date);
        }
        Rule::split => {
            let pairs = pair.into_inner();
            let txn = split::split(journal, pairs, state)?;
            println!("{}", txn);
        }
        Rule::reg => {
            let matcher = pair.into_inner().next();
            let query = matcher
                .map(|m| QueryType::MatchAccn(m.as_str().into()))
                .unwrap_or_default();
            println!("{}", journal.query(query).into_regs().join("\n"));
        }
        _ => unreachable!("unexpected rule: {:?}", pair.as_rule()),
    };

    Ok(())
}

fn parse_args() -> Result<(Args, Journal)> {
    let args = <Args as clap::Parser>::parse();
    let journal = Journal::from_file(&args.file)
        .with_context(|| format!("Failed to open journal file: {}", args.file))?;

    Ok((args, journal))
}

fn exit_gracefully(e: impl Display) -> ! {
    eprintln!("{}: {:#}", "error".red().bold(), e);
    std::process::exit(1)
}
