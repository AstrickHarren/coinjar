mod split;
mod util;

use std::str::FromStr;

use anyhow::{Context, Result};
use chrono::{Local, NaiveDate};
use clap::Parser;
use colored::Colorize;
use itertools::Itertools;
use rustyline::error::ReadlineError;

use crate::journal::{register::QueryType, Journal, Txn};

struct ReplState {
    date: NaiveDate,
    file: String,
    new_txns: Vec<Txn>,
}

#[derive(Debug, Parser)]
struct Args {
    file: String,
}

#[derive(Debug, Parser)]
#[clap(name = "")]
enum Cmd {
    #[clap(alias = "q")]
    Quit,
    #[clap(alias = "s")]
    Save,
    #[clap(alias = "reg")]
    Register {
        matcher: Option<String>,
    },
    Date {
        date: NaiveDate,
    },
    Inspect,

    #[clap(trailing_var_arg = true)]
    Split {
        args: Vec<String>,
    },
}

pub(crate) fn repl() {
    let args = Args::parse();
    let mut journal = Journal::from_file(&args.file)
        .with_context(|| {
            format!(
                "{} failed to open journal file {}",
                "error".red().bold(),
                args.file
            )
        })
        .unwrap_or_else(|e| {
            eprintln!("{:#}", e);
            std::process::exit(1);
        });

    let mut st = ReplState {
        date: Local::now().date_naive(),
        file: args.file,
        new_txns: Vec::new(),
    };

    let mut rl = rustyline::DefaultEditor::new().unwrap();
    loop {
        let ret: Result<()> = try {
            // let cmd: String = Input::new().interact_text()?;
            let cmd = rl.readline("coinjar> ");
            let cmd = match cmd {
                Err(ReadlineError::Interrupted) => continue,
                Err(ReadlineError::Eof) => return,
                cmd => cmd?,
            };
            rl.add_history_entry(cmd.as_str())?;

            let cmd =
                Cmd::try_parse_from(std::iter::once("").chain(cmd.as_str().split_whitespace()))?;

            match cmd {
                Cmd::Quit => return,
                Cmd::Register { matcher } => reg(&journal, matcher),
                Cmd::Date { date } => {
                    st.date = date;
                    println!("date: {}", date);
                }
                Cmd::Inspect => {
                    inspect(&st);
                }

                Cmd::Save => {
                    journal.save_to_file(&st.file)?;
                    st.new_txns.clear();
                }
                Cmd::Split { args } => {
                    let txn = split::split(&mut journal, args.as_slice(), &st)?;
                    println!("{}", &txn);
                    st.new_txns.push(txn.into());
                }
            }
        };

        ret.with_context(|| format!("{}", "error".red().bold()))
            .unwrap_or_else(|e| eprintln!("{:#}", e));
    }
}

fn inspect(st: &ReplState) {
    println!("date: {}", st.date);
    println!("file: {}", st.file);
    println!("txn: [+]{}", st.new_txns.len());
}

fn reg(journal: &Journal, matcher: Option<String>) {
    let query = matcher.map(QueryType::MatchAccn).unwrap_or_default();
    let query = journal.query(query);
    println!("{}", query.into_regs().format("\n"));
}
