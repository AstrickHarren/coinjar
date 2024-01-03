use std::fmt::{Display, Write};

use chrono::NaiveDate;
use indenter::indented;
use itertools::Itertools;

use super::{
    accn::{AccnId, AccnStore},
    valuable::Money,
};

#[derive(Debug)]
struct Posting {
    accn: AccnId,
    money: Money,
}

#[derive(Debug)]
struct Booking {
    date: NaiveDate,
    desc: String,
    postings: Vec<Posting>,
}

#[derive(Debug)]
struct Journal {
    accn_store: AccnStore,
    bookings: Vec<Booking>,
}

impl Posting {
    fn format(&self, accn_store: &AccnStore) -> String {
        let accn = accn_store.accn(self.accn);
        format!("{:<70}\t{}", accn.abs_name(), self.money)
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

    fn new(date: NaiveDate, desc: impl ToString) -> Self {
        Self {
            date,
            desc: desc.to_string(),
            postings: Vec::new(),
        }
    }

    fn with_posting(mut self, accn: AccnId, money: Money) -> Self {
        self.postings.push(Posting { accn, money });
        self
    }
}

impl Display for Journal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let group_by_date = self.bookings.iter().into_group_map_by(|b| b.date);

        for (date, bookings) in group_by_date.into_iter().sorted_by_key(|(date, _)| *date) {
            writeln!(f, "{}", date)?;
            for booking in bookings {
                writeln!(
                    indented(f),
                    "{}\n{}\n",
                    booking.desc,
                    booking.format_postings(&self.accn_store)
                )?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use chrono::Local;

    use super::*;
    use crate::{accn::tests::test_accn_store, valuable::Currency};

    #[test]
    fn test_journal() {
        let accn_store = test_accn_store();
        let beer = accn_store.find_accn("beer").unwrap();
        let salary = accn_store.find_accn("salary").unwrap();

        let today = Local::now().date_naive();
        let yesterday = today.pred_opt().unwrap();

        let breakfast = Booking::new(today, "Breakfast")
            .with_posting(beer.id(), Money::from_major(500, Currency::usd()))
            .with_posting(salary.id(), Money::from_major(-500, Currency::usd()));

        let lunch = Booking::new(today, "Lunch")
            .with_posting(beer.id(), Money::from_major(500, Currency::usd()))
            .with_posting(salary.id(), Money::from_major(-500, Currency::usd()));

        let dinner = Booking::new(yesterday, "Dinner")
            .with_posting(beer.id(), Money::from_major(500, Currency::usd()))
            .with_posting(salary.id(), Money::from_major(-500, Currency::usd()));

        let journal = Journal {
            accn_store,
            bookings: vec![lunch, dinner, breakfast],
        };

        println!("{}", journal)
    }
}
