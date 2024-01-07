#![allow(dead_code)]

use clap::{Parser, Subcommand};
use colored::Colorize;
use journal::{
    extension::{split::Split, NoExtension},
    Journal,
};
use tabled::{settings::Style, Table};

use crate::{fmt_table::DisplayTable, journal::query::Query};

mod accn;
mod fmt_table;
mod journal;
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

    Contact {
        #[clap(required = true)]
        name: String,
    },
}

fn main() {
    let args = Args::parse();
    let journal = Journal::from_file::<Split<NoExtension>>(&args.file_path).unwrap_or_else(|e| {
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
        Command::Contact { name } => contact_details(&journal, &name),
    }
}

fn income_statement(journal: &Journal) {
    let today = chrono::Local::now().naive_local().date();
    let week_ago = chrono::Local::now().naive_local().date() - chrono::Duration::weeks(1);
    let query = Query::new().since(week_ago).until(today);
    let income = journal.query_posting(query.clone().accn(journal.accns().income()));
    let expense = journal.query_posting(query.clone().accn(journal.accns().expense()));
    println!(
        "Income:\n{}\nExpense\n{}",
        income.daily_change().into_table(),
        expense.daily_change().into_table()
    )
}

fn contact_details(journal: &Journal, name: &str) {
    let contact = journal.accns().find_contact(name).unwrap_or_else(|| {
        eprintln!("No contact found with name: {}", name);
        std::process::exit(1);
    });
    let query = journal.query_contact(contact);
    println!(
        "{} {}\n{}",
        "Contact".purple().bold(),
        name.blue(),
        Table::new(query.balances()).with(Style::modern()),
    )
}
