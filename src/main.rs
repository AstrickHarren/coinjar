#![allow(dead_code)]

use clap::{Parser, Subcommand};
use journal::Journal;

use crate::{fmt_table::DisplayTable, journal::query::Query};

mod accn;
mod fmt_table;
mod journal;
mod parser;
mod valuable;

#[derive(Parser, Debug)]
#[command(author, version)]
struct Args {
    #[clap(long, short, default_value = "journal.coin")]
    file_path: String,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    #[clap(alias = "c")]
    Check {
        #[clap(long, short)]
        verbose: bool,
    },
    #[clap(alias = "fmt")]
    Format,
    #[clap(alias = "i")]
    IncomeStatement,
}

fn main() {
    let args = Args::parse();
    let journal = Journal::from_file(&args.file_path).unwrap_or_else(|e| {
        eprintln!("Error parsing journal: {}", e);
        std::process::exit(1);
    });

    match args.command {
        Command::Check { verbose } => {
            if verbose {
                println!("{:#}", journal);
            }
        }
        Command::Format => journal.to_file(&args.file_path),
        Command::IncomeStatement => income_statement(&journal),
    }
}

fn income_statement(journal: &Journal) {
    let week_ago = chrono::Local::now().naive_local().date() - chrono::Duration::weeks(1);
    let query = Query::new().since(week_ago);
    let income = journal.query_posting(query.clone().accn(journal.accns().income()));
    let expense = journal.query_posting(query.clone().accn(journal.accns().expense()));
    println!(
        "Income:\n{}\nExpense\n{}",
        income.daily_change().as_table(),
        expense.daily_change().as_table()
    )
}
