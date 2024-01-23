#![allow(dead_code)]
#![feature(try_blocks)]
#![feature(impl_trait_in_assoc_type)]

use anyhow::Context;
use clap::Parser;
use colored::Colorize;

use crate::transaction::Journal;

mod accn;
mod parser;
mod transaction;
mod valuable;

#[cfg(test)]
mod tests;

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value = "journal.coin")]
    file: String,
}

fn main() {
    let args = Args::parse();
    let journal = Journal::from_file(&args.file)
        .with_context(|| format!("Failed to parse journal file {}", args.file))
        .unwrap_or_else(|e| {
            eprintln!("{}: {:#}", "error".red().bold(), e);
            std::process::exit(1);
        });
    println!("{}", journal);
}
