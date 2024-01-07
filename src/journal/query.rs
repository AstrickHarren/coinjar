use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    iter::successors,
};

use chrono::NaiveDate;

use itertools::Itertools;
use tabled::Tabled;

use crate::{
    accn::{AccnId, Contact},
    journal::{Journal, Posting},
    valuable::Valuable,
};

use super::Booking;

#[derive(Clone, Debug, PartialEq)]
struct PostingQuery<'a> {
    date: NaiveDate,
    desc: &'a str,
    posting: &'a Posting,
    booking: &'a Booking,
}

#[derive(Clone, Debug, Tabled)]
pub(crate) struct BalanceQuery<'a> {
    date: NaiveDate,
    desc: &'a str,
    change: Valuable,
    balance: Valuable,
}

pub(crate) struct PostingQuerys<'a, 'b> {
    postings: Box<dyn Iterator<Item = PostingQuery<'a>> + 'b>,
    since: Option<NaiveDate>,
    until: Option<NaiveDate>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub(crate) enum Query {
    Account(AccnId),
    Accounts(Vec<AccnId>),
    Since(NaiveDate),
    Until(NaiveDate),
    And(Box<Query>, Box<Query>),
    #[default]
    All,
}

impl Query {
    pub(crate) fn new() -> Self {
        Self::All
    }

    pub(crate) fn accns(accns: impl IntoIterator<Item = impl Into<AccnId>>) -> Self {
        Self::Accounts(accns.into_iter().map(|a| a.into()).collect_vec())
    }

    pub(crate) fn accn(self, accn: impl Into<AccnId>) -> Self {
        Self::And(Box::new(self), Box::new(Self::Account(accn.into())))
    }

    pub(crate) fn since(self, date: NaiveDate) -> Self {
        Self::And(Box::new(self), Box::new(Self::Since(date)))
    }

    pub(crate) fn until(self, date: NaiveDate) -> Self {
        Self::And(Box::new(self), Box::new(Self::Until(date)))
    }
}

impl Journal {
    fn postings(&self) -> impl Iterator<Item = PostingQuery<'_>> {
        self.bookings.iter().flat_map(|b| {
            b.postings.iter().map(move |p| PostingQuery {
                date: b.date,
                desc: &b.desc,
                posting: p,
                booking: b,
            })
        })
    }

    pub(crate) fn query_posting(&self, query: Query) -> PostingQuerys<'_, '_> {
        match query {
            Query::Account(accn) => self
                .postings()
                .filter(move |p| {
                    self.accns()
                        .accn(p.posting.accn)
                        .ancesters()
                        .map(|a| a.id())
                        .contains(&accn)
                })
                .into(),

            Query::Accounts(accns) => {
                let accns: HashSet<_> = accns.into_iter().collect();
                PostingQuerys::from(self.postings().filter(move |p| {
                    self.accns()
                        .accn(p.posting.accn)
                        .ancesters()
                        .map(|a| a.id())
                        .any(|a| accns.contains(&a))
                }))
            }

            Query::Since(date) => {
                PostingQuerys::from(self.postings().filter(move |p| p.date >= date))
                    .since_date(date)
            }
            Query::Until(date) => {
                PostingQuerys::from(self.postings().filter(move |p| p.date <= date))
                    .until_date(date)
            }
            Query::And(p, q) => {
                let p = self.query_posting(*p);
                let q = self.query_posting(*q);
                let since = p.since.map(|p| p.min(q.since.unwrap_or(p))).or(q.since);
                let until = p.until.map(|p| p.max(q.until.unwrap_or(p))).or(q.until);
                let q = q.postings.collect_vec();
                PostingQuerys::new(p.postings.filter(move |p| q.contains(p)), since, until)
            }
            Query::All => self.postings().into(),
        }
    }

    pub(crate) fn query_contact(&self, contact: Contact) -> PostingQuerys {
        self.query_posting(Query::accns(contact.accns()))
    }
}

impl<'a, 'b, I> From<I> for PostingQuerys<'a, 'b>
where
    I: Iterator<Item = PostingQuery<'a>> + 'b,
{
    fn from(postings: I) -> Self {
        Self {
            postings: Box::new(postings),
            since: None,
            until: None,
        }
    }
}

impl<'a, 'b> PostingQuerys<'a, 'b> {
    fn new(
        postings: impl Iterator<Item = PostingQuery<'a>> + 'b,
        since_date: Option<NaiveDate>,
        until_date: Option<NaiveDate>,
    ) -> Self {
        Self {
            postings: Box::new(postings),
            since: since_date,
            until: until_date,
        }
    }

    pub(crate) fn daily_change(self) -> HashMap<NaiveDate, Valuable> {
        let mut ret: HashMap<_, _> = self
            .postings
            .group_by(|p| p.date)
            .into_iter()
            .map(|(date, postings)| {
                let postings: PostingQuerys<'_, '_> = postings.into();
                (date, postings.total())
            })
            .collect();

        (!ret.is_empty()).then_some(()).and_then(|_| {
            let min_date = self.since.or_else(|| ret.keys().min().copied())?;
            let max_date = self.until.or_else(|| ret.keys().max().copied())?;

            let dates = successors(Some(min_date), |d| d.succ_opt().filter(|d| *d <= max_date));
            dates.for_each(|date| {
                ret.entry(date).or_insert_with(Valuable::default);
            });
            Some(())
        });

        ret
    }

    pub(crate) fn daily_balance(self) -> impl Iterator<Item = BalanceQuery<'a>> {
        self.daily_change()
            .into_iter()
            .sorted_by_key(|(date, _)| *date)
            .scan(Valuable::default(), |balance, (date, change)| {
                *balance += change.clone();
                Some(BalanceQuery {
                    date,
                    desc: "",
                    change,
                    balance: balance.clone(),
                })
            })
    }

    pub(crate) fn balances(self) -> impl Iterator<Item = BalanceQuery<'a>> {
        self.postings
            .sorted_by_key(|p| p.date)
            .group_by(|p| p.booking)
            .into_iter()
            .scan(Valuable::default(), |balance, (b, p)| {
                let change: Valuable = p.map(|p| p.posting.money.clone()).sum();
                *balance += change.clone();
                Some(BalanceQuery {
                    date: b.date,
                    desc: &b.desc,
                    change,
                    balance: balance.clone(),
                })
            })
            .collect_vec() // TODO: remove this
            .into_iter()
    }

    fn total(self) -> Valuable {
        self.postings.map(|p| p.posting.money.clone()).sum()
    }

    fn since_date(self, date: NaiveDate) -> Self {
        Self {
            since: Some(date),
            ..self
        }
    }

    fn until_date(self, date: NaiveDate) -> Self {
        Self {
            until: Some(date),
            ..self
        }
    }
}

#[cfg(test)]
mod test {
    use chrono::Local;
    use colored::Colorize;
    use tabled::{settings::Style, Table};

    use super::*;
    use crate::{fmt_table::DisplayTable, journal::test::example_journal};

    #[test]
    fn test_query() {
        let journal = example_journal();
        let income = journal.accns().income();
        let query = journal.query_posting(Query::Account(income.id()));

        println!(
            "{} {}:\n{}",
            "Query".green().bold(),
            income.name(),
            query.daily_change().into_table()
        );
    }

    #[test]
    fn test_daily_balance() {
        let journal = example_journal();
        let income = journal.accns().income();
        let week_ago = Local::now().date_naive() - chrono::Duration::weeks(1);
        let query = journal.query_posting(Query::new().accn(income.id()).since(week_ago));

        println!(
            "{} {}:\n{}",
            "Daily Balance".green().bold(),
            income.name(),
            query.daily_change().into_table()
        );
    }

    #[test]
    fn test_balance() {
        let journal = example_journal();
        let expense = journal.accns().expense();
        let week_ago = Local::now().date_naive() - chrono::Duration::weeks(1);
        let query = journal.query_posting(Query::new().accn(expense.id()).since(week_ago));

        println!(
            "{} {}:\n{}",
            "Balance".green().bold(),
            expense.name(),
            Table::new(query.balances()).with(Style::modern()),
        );
    }
}
