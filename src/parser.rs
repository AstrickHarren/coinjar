use chrono::NaiveDate;
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

use crate::{
    accn::{AccnMut, AccnStore, ContactMut},
    journal::{Booking, Journal},
    valuable::{CurrencyStore, Money},
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
        let mut accn = self
            .find_accn_mut(pairs.next().unwrap().as_str())
            .unwrap()
            .id();

        while let Some(pair) = pairs.next() {
            let name = pair.as_str();

            let name = match pair.as_rule() {
                Rule::words => name.to_string(),
                Rule::contact => self.parse_contact(pair).name().to_string(),
                _ => unreachable!(
                    "unexpected rule {:?} in accn name {:?}",
                    pair.as_rule(),
                    name
                ),
            };

            accn = self
                .find_accn_mut(&name)
                .map(|accn| accn.id())
                .unwrap_or_else(|| self.accn_mut(accn).open_child_accn(name).id());
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
    fn parse_coinfile(mut self, file_path: &str) -> Journal {
        let file = std::fs::read_to_string(file_path).unwrap();
        let pairs = CoinParser::parse(Rule::grammar, &file).unwrap_or_else(|e| panic!("{}", e));

        for pair in pairs {
            match pair.as_rule() {
                Rule::chapter => self.parse_chapter(pair),
                Rule::EOI => (),
                _ => unreachable!(),
            }
        }

        Journal::new(self.accn_store, self.currency_store, self.bookings)
    }

    fn parse_chapter(&mut self, pair: Pair<'_, Rule>) {
        let mut pairs = pair.into_inner();
        let date = NaiveDate::parse_from_str(pairs.next().unwrap().as_str(), "%Y-%m-%d").unwrap();

        for pair in pairs {
            match pair.as_rule() {
                Rule::booking => {
                    let booking = self.parse_booking(date, pair);
                    self.bookings.push(booking);
                }
                _ => unreachable!(),
            }
        }
    }

    fn parse_booking(&mut self, date: NaiveDate, pair: Pair<'_, Rule>) -> Booking {
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
    }
}

#[cfg(test)]
mod test {

    use crate::valuable::test::example_currency_store;

    use super::*;

    #[test]
    fn parse_example() {
        let coin_path = "journal.coin";
        let parser = CoinParser {
            accn_store: AccnStore::new(),
            currency_store: example_currency_store(),
            bookings: Vec::new(),
        };
        let journal = parser.parse_coinfile(coin_path);
        println!("{}", journal);
    }
}
