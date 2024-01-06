pub(crate) mod query;

use std::fmt::{Display, Write};

use chrono::NaiveDate;
use colored::Colorize;
use indenter::indented;
use itertools::Itertools;

use crate::{accn::ContactId, valuable::CurrencyStore};

use super::{
    accn::{AccnId, AccnStore},
    valuable::Money,
};

#[derive(Debug, PartialEq)]
pub(super) struct Posting {
    accn: AccnId,
    money: Money,
}

#[derive(Debug)]
pub(crate) struct Booking {
    date: NaiveDate,
    desc: String,
    payee: ContactId,
    postings: Vec<Posting>,
}

#[derive(Debug)]
pub(crate) struct Journal {
    accn_store: AccnStore,
    currency_store: CurrencyStore,
    bookings: Vec<Booking>,
}

impl Posting {
    fn format(&self, accn_store: &AccnStore) -> String {
        let accn = accn_store.accn(self.accn);
        format!("{:<60}{:>5}", accn.abs_name(), self.money)
    }
}

impl Booking {
    fn format_postings(&self, accn_store: &AccnStore) -> String {
        self.postings
            .iter()
            .map(|p| p.format(accn_store))
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub(crate) fn new(date: NaiveDate, desc: impl ToString, payee: impl Into<ContactId>) -> Self {
        Self {
            date,
            desc: desc.to_string(),
            postings: Vec::new(),
            payee: payee.into(),
        }
    }

    pub(crate) fn with_posting(mut self, accn: impl Into<AccnId>, money: Money) -> Self {
        self.postings.push(Posting {
            accn: accn.into(),
            money,
        });
        self
    }
}

impl Journal {
    pub(crate) fn new(
        accn_store: AccnStore,
        currency_store: CurrencyStore,
        bookings: Vec<Booking>,
    ) -> Self {
        Self {
            accn_store,
            currency_store,
            bookings,
        }
    }

    pub(crate) fn accns(&self) -> &AccnStore {
        &self.accn_store
    }
}

impl Display for Journal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let group_by_date = self.bookings.iter().into_group_map_by(|b| b.date);

        if f.alternate() {
            writeln!(f, "{}\n", self.accn_store)?;
            writeln!(f, "{}", "Bookings:".purple())?;
        }

        for (date, bookings) in group_by_date.into_iter().sorted_by_key(|(date, _)| *date) {
            writeln!(f, "{}", date)?;
            for booking in bookings {
                writeln!(
                    indented(f),
                    "{} @{}\n{}\n",
                    booking.desc,
                    self.accn_store.contact(booking.payee).name(),
                    booking.format_postings(&self.accn_store)
                )?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod test {
    use chrono::Local;

    use super::*;
    use crate::{
        accn::tests::example_accn_store,
        valuable::{test::example_currency_store, Currency},
    };

    pub(crate) fn example_journal() -> Journal {
        Journal::from_file("journal.coin")
            .unwrap_or_else(|e| panic!("Error parsing journal: {}", e))
    }

    #[test]
    fn test_journal() {
        let accn_store = example_accn_store();
        let beer = accn_store.find_accn("beer").unwrap();
        let salary = accn_store.find_accn("salary").unwrap();

        let today = Local::now().date_naive();
        let yesterday = today.pred_opt().unwrap();

        let alice = accn_store.find_contact("Alice").unwrap();

        let breakfast = Booking::new(today, "Breakfast", &alice)
            .with_posting(beer.id(), Money::from_major(500, Currency::usd()))
            .with_posting(salary.id(), Money::from_major(-500, Currency::usd()));

        let lunch = Booking::new(today, "Lunch", &alice)
            .with_posting(beer.id(), Money::from_major(500, Currency::usd()))
            .with_posting(salary.id(), Money::from_major(-500, Currency::usd()));

        let dinner = Booking::new(yesterday, "Dinner", &alice)
            .with_posting(beer.id(), Money::from_major(500, Currency::usd()))
            .with_posting(salary.id(), Money::from_major(-500, Currency::usd()));

        let journal = Journal {
            accn_store,
            bookings: vec![lunch, dinner, breakfast],
            currency_store: example_currency_store(),
        };

        println!("{}", journal)
    }
}
