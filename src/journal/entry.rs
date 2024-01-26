use std::fmt::{Debug, Display};

use itertools::Itertools;

use crate::{accn::AccnEntry, valuable::MoneyEntry};

use super::*;

#[derive(Debug, Clone, Copy)]
pub(crate) struct PostingEntry<'a> {
    posting: Posting,
    journal: &'a Journal,
}

impl<'a> PostingEntry<'a> {
    pub(super) fn accn(self) -> AccnEntry<'a> {
        self.data().accn.into_accn(&self.journal.accns)
    }

    fn data(self) -> &'a PostingData {
        &self.journal.txns.postings[&self.posting]
    }

    pub(super) fn txn(self) -> TxnEntry<'a> {
        self.data().txn.into_txn(self.journal)
    }

    pub(super) fn money(self) -> MoneyEntry<'a> {
        self.data().money.into_money(&self.journal.currencies)
    }
}

impl Posting {
    pub(super) fn into_posting(self, journal: &Journal) -> PostingEntry<'_> {
        PostingEntry {
            posting: self,
            journal,
        }
    }
}

impl Txn {
    pub(crate) fn into_txn(self, journal: &Journal) -> TxnEntry<'_> {
        TxnEntry { txn: self, journal }
    }
}

impl Display for PostingEntry<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "    {:<60}{:>10}",
            self.accn(),
            self.data().money.fmt(&self.journal.currencies)
        )
    }
}

#[derive(Debug)]
pub(crate) struct TxnEntry<'a> {
    txn: Txn,
    journal: &'a Journal,
}

impl<'a> TxnEntry<'a> {
    fn data(&self) -> &TxnData {
        &self.journal.txns.txns[&self.txn]
    }

    pub(super) fn date(&self) -> NaiveDate {
        self.data().date
    }

    pub(super) fn desc(&self) -> &str {
        &self.data().description
    }

    fn postings(&self) -> impl Iterator<Item = PostingEntry<'_>> {
        self.data()
            .postings
            .iter()
            .map(move |posting| PostingEntry {
                posting: *posting,
                journal: self.journal,
            })
    }

    pub(super) fn new(txn: Txn, journal: &'a Journal) -> Self {
        Self { txn, journal }
    }
}

impl From<TxnEntry<'_>> for Txn {
    fn from(entry: TxnEntry) -> Self {
        entry.txn
    }
}

impl Display for TxnEntry<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}\n{}",
            self.data().date,
            self.data().description,
            self.postings().join("\n")
        )
    }
}
