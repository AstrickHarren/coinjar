use std::io::Write;

use chrono::{Local, NaiveDate};
use pest::{
    error::{Error, ErrorVariant},
    iterators::Pair,
    Parser,
};
use pest_derive::Parser;

use crate::{
    accn::{AccnMut, AccnStore, ContactMut},
    journal::{Booking, Journal},
    valuable::{test::example_currency_store, CurrencyStore, Money},
};

#[derive(Debug, Parser)]
#[grammar = "../share/grammar.pest"]
struct CoinParser {
    accn_store: AccnStore,
    currency_store: CurrencyStore,
    bookings: Vec<Booking>,
}

impl AccnStore {
    fn parse_accn(&mut self, pair: Pair<'_, Rule>) -> AccnMut<'_> {
        let mut pairs = pair.into_inner();
        let mut accn = self.root(pairs.next().unwrap().as_str()).unwrap().id();

        while let Some(pair) = pairs.next() {
            let name = pair.as_str();

            let name = match pair.as_rule() {
                Rule::words => name.to_string(),
                Rule::contact => {
                    let mut contact = self.parse_contact(pair);
                    contact.make_accns();
                    name.to_string()
                }
                _ => unreachable!(
                    "unexpected rule {:?} in accn name {:?}",
                    pair.as_rule(),
                    name
                ),
            };

            accn = self.accn_mut(accn).child_entry(name).or_open().id()
        }

        self.accn_mut(accn)
    }

    fn parse_contact(&mut self, pair: Pair<'_, Rule>) -> ContactMut {
        let name = pair.as_str();
        debug_assert!(name.starts_with("@"));

        let name = &name[1..];
        let id = self
            .find_contact_mut(name)
            .map(|contact| contact.id())
            .unwrap_or_else(|| self.add_contact(name).id());
        self.contact_mut(id)
    }
}

impl CoinParser {
    fn parse_coinfile(mut self, file_path: &str) -> Result<Journal, String> {
        let file = std::fs::read_to_string(file_path).unwrap();
        let pairs = CoinParser::parse(Rule::grammar, &file).map_err(|e| format!("{}", e))?;

        for pair in pairs {
            match pair.as_rule() {
                Rule::chapter => self.parse_chapter(pair)?,
                Rule::EOI => (),
                _ => unreachable!(),
            }
        }

        Ok(Journal::new(
            self.accn_store,
            self.currency_store,
            self.bookings,
        ))
    }

    fn parse_date(&mut self, pair: Pair<'_, Rule>) -> NaiveDate {
        let date = pair.as_str();
        match date {
            "today" => Local::now().date_naive(),
            "yesterday" => Local::now().date_naive().pred_opt().unwrap(),
            _ => NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap(),
        }
    }

    fn parse_chapter(&mut self, pair: Pair<'_, Rule>) -> Result<(), String> {
        let mut pairs = pair.into_inner();
        let date = self.parse_date(pairs.next().unwrap());

        for pair in pairs {
            match pair.as_rule() {
                Rule::booking => {
                    let booking = self.parse_booking(date, pair)?;
                    self.bookings.push(booking);
                }
                _ => unreachable!(),
            }
        }

        Ok(())
    }

    fn parse_booking(&mut self, date: NaiveDate, pair: Pair<'_, Rule>) -> Result<Booking, String> {
        let span = pair.as_span();
        let mut pairs = pair.into_inner();
        let desc = pairs.next().unwrap().as_str();
        let contact = self.accn_store.parse_contact(pairs.next().unwrap());

        let mut booking = Booking::new(date, desc, contact);
        for pair in pairs {
            match pair.as_rule() {
                Rule::posting => {
                    let mut pairs = pair.into_inner();
                    let accn = self.accn_store.parse_accn(pairs.next().unwrap());
                    let money =
                        Money::from_str(pairs.next().unwrap().as_str(), &self.currency_store);
                    booking = booking.with_posting(accn.id(), money);
                }
                _ => unreachable!(),
            }
        }
        booking
            .is_balanced()
            .then_some(booking)
            .ok_or_else(|| pest_custom_err(span, "booking not balanced").to_string())
    }
}

impl Journal {
    pub(crate) fn from_file(file_path: &str) -> Result<Self, String> {
        let parser = CoinParser {
            accn_store: AccnStore::new(),
            currency_store: example_currency_store(),
            bookings: Vec::new(),
        };
        parser.parse_coinfile(file_path)
    }

    pub(crate) fn to_file(&self, file_path: &str) {
        let string = self.to_string();
        let mut file = std::fs::File::create(file_path).unwrap();
        file.write_all(string.as_bytes()).unwrap();
    }
}

fn pest_custom_err(span: pest::Span<'_>, msg: impl ToString) -> Error<Rule> {
    Error::new_from_span(
        ErrorVariant::<Rule>::CustomError {
            message: msg.to_string(),
        },
        span,
    )
}

#[cfg(test)]
mod test {

    use crate::valuable::test::example_currency_store;

    use super::*;

    #[test]
    fn parse_example() -> Result<(), String> {
        let coin_path = "./test/example.coin";
        let parser = CoinParser {
            accn_store: AccnStore::new(),
            currency_store: example_currency_store(),
            bookings: Vec::new(),
        };
        let journal = parser.parse_coinfile(coin_path)?;
        println!("{}", journal.accns());
        println!("{}", journal);
        Ok(())
    }

    #[test]
    fn reparse_example() -> Result<(), String> {
        let ref_journal = "./test/example.coin";
        let reparse_journal = "./target/example.coin";

        let ref_journal = Journal::from_file(ref_journal)?;
        ref_journal.to_file(reparse_journal);

        let journal = Journal::from_file(reparse_journal)?;
        assert_eq!(ref_journal.to_string(), journal.to_string());
        Ok(())
    }
}
