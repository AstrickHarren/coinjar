use std::{collections::HashMap, fmt::Debug};

use chrono::NaiveDate;

use itertools::Itertools;

use crate::{
    accn::AccnId,
    journal::{Journal, Posting},
    valuable::Valuable,
};

#[derive(Clone, Debug, PartialEq)]
struct PostingQuery<'a> {
    date: NaiveDate,
    desc: &'a str,
    posting: &'a Posting,
}

pub(crate) struct PostingQuerys<'a, 'b> {
    postings: Box<dyn Iterator<Item = PostingQuery<'a>> + 'b>,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub(crate) enum Query {
    Account(AccnId),
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
            })
        })
    }

    pub(crate) fn query_posting(&self, query: Query) -> PostingQuerys<'_, '_> {
        match query {
            Query::Account(accn) => self
                .postings()
                .filter(move |p| {
                    self.accn_store()
                        .accn(p.posting.accn)
                        .ancesters()
                        .map(|a| a.id())
                        .contains(&accn)
                })
                .into(),

            Query::Since(date) => self.postings().filter(move |p| p.date >= date).into(),
            Query::Until(date) => self.postings().filter(move |p| p.date <= date).into(),
            Query::And(p, q) => {
                let p = self.query_posting(*p);
                let q = self.query_posting(*q).postings.collect_vec();
                let postings = p.postings.filter(move |p| q.contains(p));
                postings.into()
            }
            Query::All => self.postings().into(),
        }
    }
}

impl<'a, 'b, I> From<I> for PostingQuerys<'a, 'b>
where
    I: Iterator<Item = PostingQuery<'a>> + 'b,
{
    fn from(postings: I) -> Self {
        Self {
            postings: Box::new(postings),
        }
    }
}

impl<'a, 'b> PostingQuerys<'a, 'b> {
    pub(crate) fn daily_change(self) -> HashMap<NaiveDate, Valuable> {
        self.postings
            .group_by(|p| p.date)
            .into_iter()
            .map(|(date, postings)| {
                let postings: PostingQuerys<'_, '_> = postings.into();
                (date, postings.total())
            })
            .collect()
    }

    fn total(self) -> Valuable {
        self.postings.fold(Valuable::default(), |mut acc, p| {
            acc.add_money(p.posting.money.clone());
            acc
        })
    }
}

#[cfg(test)]
mod test {
    use colored::Colorize;

    use super::*;
    use crate::{fmt_table::DisplayTable, journal::test::example_journal};

    #[test]
    fn test_query() {
        let journal = example_journal();
        let income = journal.accn_store().income();
        let query = journal.query_posting(Query::Account(income.id()));

        println!(
            "{} {}:\n{}",
            "Query".green().bold(),
            income.name(),
            query.daily_change().as_table()
        );
    }
}
