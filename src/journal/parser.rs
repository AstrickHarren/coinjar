use std::io::Write;

use chrono::{Local, NaiveDate};
use pest::{
    error::{Error, ErrorVariant},
    iterators::Pair,
    Parser,
};
use pest_derive::Parser;
use uuid::Uuid;

use crate::{
    accn::{AccnId, AccnMut, AccnStore, ContactId, ContactMut},
    valuable::{CurrencyStore, Money, Valuable},
};

use super::{Booking, Journal, Posting};

#[derive(Debug, Parser)]
#[grammar = "../share/grammar.pest"]
struct CoinParser {
    accn_store: AccnStore,
    currency_store: CurrencyStore,
    bookings: Vec<Booking>,
}

struct InferredBookingBuilder {
    date: NaiveDate,
    desc: String,
    payee: ContactId,
    postings: Vec<Posting>,
    inferred_posting: Option<AccnId>,
}

impl InferredBookingBuilder {
    fn from_booking(booking: Booking) -> Self {
        Self {
            date: booking.date,
            desc: booking.desc,
            payee: booking.payee,
            postings: booking.postings,
            inferred_posting: None,
        }
    }

    fn with_posting(mut self, accn: impl Into<AccnId>, money: Money) -> Self {
        self.postings.push(Posting {
            accn: accn.into(),
            money,
        });
        self
    }

    fn with_inferred_posting(mut self, accn: impl Into<AccnId>) -> Option<Self> {
        match self.inferred_posting {
            Some(_) => None,
            None => {
                self.inferred_posting = Some(accn.into());
                Some(self)
            }
        }
    }

    fn inbalance(&self) -> Valuable {
        self.postings.iter().map(|p| -p.money.clone()).sum()
    }

    fn into_booking(mut self) -> Booking {
        self.postings.extend(
            self.inferred_posting
                .map(|accn| {
                    self.inbalance()
                        .into_moneys()
                        .map(move |money| Posting { accn, money })
                })
                .into_iter()
                .flatten(),
        );

        Booking {
            id: Uuid::new_v4(),
            date: self.date,
            desc: self.desc,
            payee: self.payee,
            postings: self.postings,
        }
    }
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
        debug_assert!(name.starts_with("@"));

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

impl CoinParser {
    fn parse_coinfile(mut self, file_path: &str) -> Result<Journal, String> {
        let file = std::fs::read_to_string(file_path).unwrap();
        let pairs = CoinParser::parse(Rule::grammar, &file).map_err(|e| format!("{}", e))?;

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

        let booking = Booking::new(date, desc, contact);
        let mut booking = InferredBookingBuilder::from_booking(booking);
        for pair in pairs {
            match pair.as_rule() {
                Rule::posting => {
                    let mut pairs = pair.into_inner();
                    let accn = self.accn_store.parse_accn(pairs.next().unwrap());
                    match pairs.next() {
                        Some(money) => {
                            booking = booking.with_posting(
                                accn.id(),
                                Money::from_str(money.as_str(), &self.currency_store).ok_or_else(
                                    || {
                                        pest_custom_err(money.as_span(), "Currency not found")
                                            .to_string()
                                    },
                                )?,
                            );
                        }
                        None => {
                            booking =
                                booking.with_inferred_posting(accn.id()).ok_or_else(|| {
                                    pest_custom_err(span, "cannot have multiple inferred posting")
                                        .to_string()
                                })?;
                        }
                    }
                }
                _ => unreachable!(),
            }
        }

        let booking = booking.into_booking();
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
            currency_store: CurrencyStore::new(),
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

    use super::*;

    #[test]
    fn parse_currency() {
        let coin = stringify!(
            currency USD $ -- US Dollar
        );
        let mut currency_store = CurrencyStore::new();
        let pairs = CoinParser::parse(Rule::currencies, coin).unwrap_or_else(|e| panic!("{}", e));
        for pair in pairs {
            currency_store.parse_currency(pair);
        }

        println!("{}", currency_store);
    }

    #[test]
    fn parse_pairs() {
        let file = std::fs::read_to_string("./test/example.coin").unwrap();
        let pairs = CoinParser::parse(Rule::grammar, &file).unwrap_or_else(|e| panic!("{}", e));
        dbg!(pairs);
    }

    #[test]
    fn parse_example() {
        let coin_path = "./test/example.coin";
        let parser = CoinParser {
            accn_store: AccnStore::new(),
            currency_store: CurrencyStore::new(),
            bookings: Vec::new(),
        };
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

        let ref_journal = Journal::from_file(ref_journal)?;
        ref_journal.to_file(reparse_journal);

        let journal = Journal::from_file(reparse_journal)?;
        assert_eq!(ref_journal.to_string(), journal.to_string());
        Ok(())
    }
}
