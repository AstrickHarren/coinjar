mod date;
mod split;
mod util;

use std::fmt::Display;

use anyhow::{anyhow, bail, Context, Result};
use chrono::{Local, NaiveDate};
use colored::Colorize;
use inquire::Select;
use itertools::Itertools;
use pest::Parser;
use rustyline::{config::Configurer, error::ReadlineError};

use crate::{
    journal::{
        parser::{IdentParser, Rule},
        register::QueryType,
        Journal, Txn,
    },
    util::NotEmpty,
};

use self::{date::DateArg, util::fuzzy_create_accn};

struct ReplState {
    date: NaiveDate,
    file: String,
    new_txns: Vec<Txn>,
    del_txns: usize,

    history_writes: Vec<Vec<Txn>>,
}

impl ReplState {
    fn inspect(&self) {
        println!("date: {}", self.date);
        println!("file: {}", self.file);
        println!(
            "changes not saved {}[+] {}[-]",
            self.new_txns.len(),
            self.del_txns
        );
    }
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
        del_txns: 0,
        history_writes: Vec::new(),
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
            state.new_txns.push(txn.into());
        }
        Rule::reg => {
            let matcher = pair.into_inner().next();
            let query = matcher
                .map(|m| QueryType::MatchAccn(m.as_str().into()))
                .unwrap_or_default();
            println!("{}", journal.query(query).into_regs().join("\n"));
        }
        Rule::accn_cmd => {
            println!("{}", journal.accns());
        }
        Rule::open => {
            let matcher = pair.into_inner().next().unwrap().as_str();
            journal
                .accns()
                .by_name_fuzzy(matcher)
                .empty()
                .map_err(|e| {
                    anyhow!(
                        "accns already exist:\n{}",
                        e.map(|accn| accn.abs_name()).join("\n")
                    )
                })?;
            let accn = fuzzy_create_accn(journal, matcher)?;
            println!("created accn: {}", accn.as_ref().abs_name());
        }
        Rule::save => {
            journal.save_to_file(&state.file)?;
            println!("saved {} txns to {}", state.new_txns.len(), state.file);
            if state.new_txns.is_empty() {
                return Ok(());
            }
            state.del_txns = 0;
            state
                .history_writes
                .push(std::mem::take(&mut state.new_txns));
        }
        Rule::undo => {
            let history = state
                .history_writes
                .pop()
                .ok_or_else(|| anyhow!("no history to undo"))?;
            println!("undo {} txns", history.len());
            for txn in history {
                journal.txn_mut(txn).remove()
            }
            journal.save_to_file(&state.file)?;
        }
        Rule::del => {
            let txns: Vec<_> = journal.txns().map(|t| t.brief()).collect();
            if txns.is_empty() {
                bail!("no transaction left to delete")
            }
            let prompt = format!("{}", "select to delete".red());
            let txn = Select::new(&prompt, txns).prompt()?.id();

            state.del_txns += 1;
            state.new_txns.retain(|t| *t != txn);
            txn.into_mut(journal).remove();
        }
        Rule::inspect => state.inspect(),
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
