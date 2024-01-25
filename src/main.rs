#![allow(dead_code)]
#![feature(try_blocks)]
#![feature(impl_trait_in_assoc_type)]
#![feature(trait_alias)]

use anyhow::{anyhow, Context, Result};
use chrono::{Local, NaiveDate};
use clap::{Parser, Subcommand};
use colored::Colorize;
use itertools::Itertools;

use crate::{
    accn::AccnEntry,
    journal::{register::QueryType, Journal},
};

mod accn;
mod journal;
mod valuable;

#[cfg(test)]
mod tests;

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value = "journal.coin")]
    file: String,

    #[clap(short, long)]
    date: Option<String>,

    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Default)]
enum Command {
    #[clap(about = "Print the register", alias = "reg")]
    Register { matcher: Option<String> },

    Transfer {
        from: String,
        to: String,
        money: String,
        desc: String,
    },

    #[default]
    Check,
}

#[derive(Parser)]
struct TransferArgs {
    from: String,
    to: String,
    money: String,
    desc: String,
}

fn main() {
    let ret: Result<()> = try {
        let args = Args::parse();
        let date: Option<NaiveDate> = args.date.map(|d| d.parse()).transpose()?;
        let mut journal = check(&args.file)?;
        let cmd = args.command.unwrap_or_default();

        match cmd {
            Command::Register { matcher } => print_register(&journal, matcher),
            Command::Transfer {
                from,
                to,
                money,
                desc,
            } => transfer(&mut journal, desc, from, to, money, date)?,
            Command::Check => (),
        }
    };

    exit_gracefully(ret);
}

fn check(file: &str) -> Result<Journal> {
    Journal::from_file(file).with_context(|| format!("Failed to parse journal file {}", file))
}

fn print_register(journal: &Journal, matcher: Option<String>) {
    let query = matcher.map_or(QueryType::All, QueryType::MatchAccn);
    let postings = journal.query(query);
    println!("{}", postings.into_regs().join("\n"));
}

fn transfer(
    journal: &mut Journal,
    desc: String,
    from: String,
    to: String,
    money: String,
    date: Option<NaiveDate>,
) -> Result<()> {
    let date = date.unwrap_or_else(|| Local::now().date_naive());
    let money = journal.parse_money(&money)?.money();

    fn accn_by_name(journal: &Journal, name: String) -> Result<AccnEntry> {
        let accn = journal.accns().by_name_unique(&name).map_err(|mut cond| {
            anyhow!(
                "cannot find a unique account named '{}', got candidates:\n{}",
                name,
                cond.join("\n")
            )
        })?;
        Ok(accn)
    }
    let from = accn_by_name(journal, from)?.id();
    let to = accn_by_name(journal, to)?.id();
    let txn = journal
        .new_txn(date, desc)
        .with_posting(from, Some(money))
        .with_posting(to, Some(-money))
        .build()?;

    println!(
        "{} building transcation\n{}",
        "Finished".green().bold(),
        txn
    );
    Ok(())
}

fn exit_gracefully<T>(result: Result<T>) -> T {
    result
        .with_context(|| format!("{}", "error".bold().red()))
        .unwrap_or_else(|e| {
            eprintln!("{:#}", e);
            std::process::exit(1);
        })
}
