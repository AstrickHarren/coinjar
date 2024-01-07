use std::{io::Write, marker::PhantomData};

use chrono::{Local, NaiveDate};
use pest::{
    error::{Error, ErrorVariant},
    iterators::Pair,
    Parser,
};
use pest_derive::Parser;

use crate::{
    accn::{AccnMut, AccnStore, ContactMut},
    valuable::{CurrencyStore, Money},
};

use super::{
    extension::{BuildBook, NoExtension},
    Booking, Journal,
};

#[derive(Parser)]
#[grammar = "../share/grammar.pest"]
struct IdentParser;

#[derive(Debug)]
struct CoinParser<B: BuildBook = NoExtension> {
    accn_store: AccnStore,
    currency_store: CurrencyStore,
    bookings: Vec<Booking>,
    booking_builder: PhantomData<B>,
}

impl AccnStore {
    fn parse_accn(&mut self, pair: Pair<'_, Rule>) -> AccnMut<'_> {
        let mut pairs = pair.into_inner();
        let mut accn = self.root(pairs.next().unwrap().as_str()).unwrap().id();

        for pair in pairs {
            let name = pair.as_str();

            let name = match pair.as_rule() {
                Rule::words => name.to_string(),
                Rule::contact => {
                    self.parse_contact(pair);
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
        debug_assert!(name.starts_with('@'));

        let name = &name[1..];
        let id = self
            .find_contact_mut(name)
            .map(|contact| contact.id())
            .unwrap_or_else(|| self.add_contact(name).id());
        self.contact_mut(id)
    }
}

impl CurrencyStore {
    fn parse_currency(&mut self, pair: Pair<'_, Rule>) {
        let mut pairs = pair.into_inner();
        let code = pairs.next().unwrap().as_str();
        let symbol = pairs
            .peek()
            .filter(|p| p.as_rule() == Rule::symbol)
            .map(|_| pairs.next().unwrap().as_str());
        let desc = pairs.next().map(|p| p.as_str());
        self.add_currency(desc, symbol, code);
    }
}

impl<B: BuildBook> CoinParser<B> {
    fn new() -> Self {
        Self {
            accn_store: AccnStore::new(),
            currency_store: CurrencyStore::new(),
            bookings: Vec::new(),
            booking_builder: PhantomData,
        }
    }

    fn parse_coinfile(mut self, file_path: &str) -> Result<Journal, String> {
        let file = std::fs::read_to_string(file_path).unwrap();
        let pairs = IdentParser::parse(Rule::grammar, &file).map_err(|e| format!("{}", e))?;

        for pair in pairs {
            match pair.as_rule() {
                Rule::chapter => self.parse_chapter(pair)?,
                Rule::currency => self.currency_store.parse_currency(pair),
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

        let mut booking = B::from_booking(Booking::new(date, desc, contact));
        for pair in pairs {
            match pair.as_rule() {
                Rule::posting => {
                    let mut pairs = pair.into_inner();
                    let accn = self.accn_store.parse_accn(pairs.next().unwrap());
                    let money = pairs
                        .next()
                        .map(|p| {
                            Money::from_str(p.as_str(), &self.currency_store).ok_or_else(|| {
                                pest_custom_err(p.as_span(), "Currency not found").to_string()
                            })
                        })
                        .transpose()?;
                    booking.with_posting(accn.as_ref(), money);
                }
                Rule::tag => {
                    let mut pairs = pair.into_inner();
                    let tag_name = pairs.next().unwrap().as_str();
                    let args = pairs.map(|p| p.as_str());
                    booking.with_tag(&mut self.accn_store, tag_name, args);
                }
                _ => unreachable!(),
            }
        }

        let booking = booking.into_booking_with(&mut self.accn_store);
        booking
            .is_balanced()
            .then_some(booking)
            .ok_or_else(|| pest_custom_err(span, "booking not balanced").to_string())
    }
}

impl Journal {
    pub(crate) fn from_file<B: BuildBook>(file_path: &str) -> Result<Self, String> {
        let parser: CoinParser<B> = CoinParser {
            accn_store: AccnStore::new(),
            currency_store: CurrencyStore::new(),
            bookings: Vec::new(),
            booking_builder: PhantomData,
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

    use super::*;

    #[test]
    fn parse_currency() {
        let coin = stringify!(
            currency USD $ -- US Dollar
        );
        let mut currency_store = CurrencyStore::new();
        let pairs = IdentParser::parse(Rule::currencies, coin).unwrap_or_else(|e| panic!("{}", e));
        for pair in pairs {
            currency_store.parse_currency(pair);
        }

        println!("{}", currency_store);
    }

    #[test]
    fn parse_pairs() {
        let file = std::fs::read_to_string("./test/example.coin").unwrap();
        let pairs = IdentParser::parse(Rule::grammar, &file).unwrap_or_else(|e| panic!("{}", e));
        dbg!(pairs);
    }

    #[test]
    fn parse_example() {
        let coin_path = "./test/example.coin";
        let parser = CoinParser::<NoExtension>::new();
        let journal = parser
            .parse_coinfile(coin_path)
            .unwrap_or_else(|e| panic!("Error parsing journal: {}", e));
        println!("{}\n", journal.accns());
        println!("{}", journal);
    }

    #[test]
    fn reparse_example() -> Result<(), String> {
        let ref_journal = "./test/example.coin";
        let reparse_journal = "./target/example.coin";

        let ref_journal = Journal::from_file::<NoExtension>(ref_journal)?;
        ref_journal.to_file(reparse_journal);

        let journal = Journal::from_file::<NoExtension>(reparse_journal)?;
        assert_eq!(ref_journal.to_string(), journal.to_string());
        Ok(())
    }
}
