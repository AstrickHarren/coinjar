use std::{
    fmt::{Debug, Display},
    ops::Deref,
};

use itertools::Itertools;

use crate::{
    accn::AccnEntry,
    valuable::{MoneyEntry, ValuableEntry},
};

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

    pub(crate) fn id(&self) -> Txn {
        self.txn
    }

    pub(crate) fn brief(self) -> TxnEntryBrief<'a> {
        TxnEntryBrief { entry: self }
    }

    fn income_statement(&self) -> impl Iterator<Item = PostingEntry<'_>> {
        let inc = self.journal.accns().income();
        let exp = self.journal.accns().expense();
        self.postings()
            .filter(move |p| p.accn().is_descendent_of(inc) || p.accn().is_descendent_of(exp))
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

pub(crate) struct TxnEntryBrief<'a> {
    entry: TxnEntry<'a>,
}

impl Display for TxnEntryBrief<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let txn = &self.entry;
        let valuable: ValuableEntry = self.entry.income_statement().map(|p| p.money()).sum();
        write!(
            f,
            "{} {:<50} {:>20}",
            txn.data().date,
            txn.data().description,
            -valuable
        )
    }
}

impl<'a> Deref for TxnEntryBrief<'a> {
    type Target = TxnEntry<'a>;
    fn deref(&self) -> &Self::Target {
        &self.entry
    }
}

pub(crate) struct TxnEntryMut<'a> {
    txn: Txn,
    journal: &'a mut Journal,
}

impl<'a> TxnEntryMut<'a> {
    pub(super) fn new(txn: Txn, journal: &'a mut Journal) -> Self {
        Self { txn, journal }
    }

    pub(crate) fn remove(self) {
        self.journal.txns.remove(self.txn);
    }
}
