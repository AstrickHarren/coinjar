use std::fmt::Display;

use chrono::NaiveDate;
use itertools::Itertools;

use crate::valuable::ValuableEntry;

use super::{entry::PostingEntry, Journal};

trait PostingIterator<'a> = Iterator<Item = PostingEntry<'a>> + 'a;

pub(crate) struct PostingQuery<'a> {
    postings: Box<dyn PostingIterator<'a> + 'a>,
}

impl<'a> PostingQuery<'a> {
    fn new(postings: impl PostingIterator<'a> + 'a) -> Self {
        Self {
            postings: Box::new(postings),
        }
    }

    pub(crate) fn into_regs(self) -> impl Iterator<Item = RegisterRow> + 'a {
        let init_bal = ValuableEntry::default();
        self.postings
            .sorted_by_key(|p| p.txn().date())
            .scan(init_bal, |bal, p| {
                *bal += p.money();
                RegisterRow {
                    date: p.txn().date(),
                    desc: p.txn().desc().to_string(),
                    accn: p.accn().to_string(),
                    change: p.money().to_string(),
                    total: bal.to_string(),
                }
                .into()
            })
    }
}

impl<'a, I> From<I> for PostingQuery<'a>
where
    I: PostingIterator<'a>,
{
    fn from(postings: I) -> Self {
        Self::new(postings)
    }
}

#[derive(Debug)]
pub(crate) struct RegisterRow {
    date: NaiveDate,
    desc: String,
    accn: String,
    change: String,
    total: String,
}

impl Display for RegisterRow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:<15} {:<40} {:<30} {:>10} {:>30}",
            self.date.format("%Y/%m/%d"),
            self.desc,
            self.accn,
            self.change,
            self.total,
        )
    }
}

#[derive(Debug, Default)]
pub(crate) enum QueryType {
    #[default]
    All,
}

impl Journal {
    pub(crate) fn query(&self, query: QueryType) -> PostingQuery {
        match query {
            QueryType::All => self
                .txns
                .postings
                .keys()
                .map(|p| p.into_posting(self))
                .into(),
        }
    }
}
