#![allow(dead_code)]

use clap::Parser;
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
    #[arg(short, long)]
    journal_file: String,
}

fn main() {
    let args = Args::parse();
    let journal = Journal::from_file(&args.journal_file);
    println!("{}", journal);

    let week_ago = chrono::Utc::now().naive_utc().date() - chrono::Duration::weeks(1);
    let query = Query::new().since(week_ago);

    let income = journal.query_posting(query.clone().accn(journal.accn_store().income()));
    let expense = journal.query_posting(query.clone().accn(journal.accn_store().expense()));

    println!(
        "Income Statement: \nIncome:\n{}\nExpense\n{}",
        income.daily_change().as_table(),
        expense.daily_change().as_table()
    )
}
