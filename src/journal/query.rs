use std::collections::HashMap;

use chrono::NaiveDate;

use itertools::Itertools;

use crate::{
    accn::AccnId,
    journal::{Journal, Posting},
    valuable::Valuable,
};

#[derive(Clone, Debug)]
struct PostingQuery<'a> {
    date: NaiveDate,
    desc: &'a str,
    posting: &'a Posting,
}

#[derive(Clone, Debug)]
struct PostingQuerys<'a, I>
where
    I: Iterator<Item = PostingQuery<'a>>,
{
    postings: I,
}

enum Query {
    Account(AccnId),
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

    fn query_posting(
        &self,
        query: Query,
    ) -> PostingQuerys<'_, impl Iterator<Item = PostingQuery<'_>>> {
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
        }
    }
}

impl<'a, I> From<I> for PostingQuerys<'a, I>
where
    I: Iterator<Item = PostingQuery<'a>>,
{
    fn from(postings: I) -> Self {
        Self { postings }
    }
}

impl<'a, I> PostingQuerys<'a, I>
where
    I: Iterator<Item = PostingQuery<'a>>,
{
    fn daily_change(self) -> HashMap<NaiveDate, Valuable> {
        self.postings
            .group_by(|p| p.date)
            .into_iter()
            .map(|(date, postings)| {
                let postings: PostingQuerys<'_, _> = postings.into();
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
