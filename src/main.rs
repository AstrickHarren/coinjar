#![allow(dead_code)]
#![feature(try_blocks)]
#![feature(impl_trait_in_assoc_type)]
#![feature(trait_alias)]

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use itertools::Itertools;

use crate::journal::{register::QueryType, Journal};

mod accn;
mod journal;
mod valuable;

#[cfg(test)]
mod tests;

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value = "journal.coin")]
    file: String,

    #[clap(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Default)]
enum Command {
    #[clap(about = "Print the register", alias = "reg")]
    Register { matcher: Option<String> },

    #[default]
    Check,
}

fn main() {
    let ret: Result<()> = try {
        let args = Args::parse();
        let journal = check(&args.file)?;
        let cmd = args.command.unwrap_or_default();

        match cmd {
            Command::Register { matcher } => print_register(&journal, matcher),
            Command::Check => (),
        }
    };

    exit_gracefully(ret);
}

fn check(file: &str) -> Result<Journal> {
    Journal::from_file(file).with_context(|| format!("Failed to parse journal file {}", file))
}

fn print_register(journal: &Journal, matcher: Option<String>) {
    let query = matcher.map_or(QueryType::All, |m| QueryType::MatchAccn(m));
    let postings = journal.query(query);
    println!("{}", postings.into_regs().join("\n"));
}

fn exit_gracefully<T>(result: Result<T>) -> T {
    result
        .with_context(|| format!("{}", "error".bold().red()))
        .unwrap_or_else(|e| {
            eprintln!("{:#}", e);
            std::process::exit(1);
        })
}
