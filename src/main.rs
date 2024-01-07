#![allow(dead_code)]

use clap::{Parser, Subcommand};
use colored::Colorize;
use journal::{
    extension::{split::Split, NoExtension},
    Journal,
};
use tabled::{settings::Style, Table};

use crate::journal::{extension::relative_date::RelativeDate, query::Query};

mod accn;
mod fmt_table;
#[macro_use]
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
        #[clap(
            long,
            short,
            default_value = "false",
            help = "Show all transactions, by default only shows Payable and Receivable from the contact"
        )]
        all: bool,
    },
}

fn main() {
    type Extension = allow_extensions!(Split, RelativeDate);
    let args = Args::parse();
    let journal = Journal::from_file::<Extension>(&args.file_path).unwrap_or_else(|e| {
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
        Command::Contact { name, all } => contact_details(&journal, &name, all),
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
        Table::new(income.daily_balance()).with(Style::modern_rounded()),
        Table::new(expense.daily_balance()).with(Style::modern_rounded()),
    )
}

fn contact_details(journal: &Journal, name: &str, show_all: bool) {
    let contact = journal.accns().find_contact(name).unwrap_or_else(|| {
        eprintln!("No contact found with name: {}", name);
        std::process::exit(1);
    });

    let query = if show_all {
        journal.query_posting(Query::accns(contact.accns()))
    } else {
        journal.query_posting(Query::accns(
            contact.payable().into_iter().chain(contact.receivable()),
        ))
    };

    println!(
        "{} {}\n{}",
        "Contact".purple().bold(),
        name.blue(),
        Table::new(query.balances()).with(Style::modern()),
    )
}
