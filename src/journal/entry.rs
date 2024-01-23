use std::fmt::{Debug, Display};

use itertools::Itertools;

use crate::accn::AccnEntry;

use super::*;

pub(crate) struct PostingEntry<'a> {
    posting: Posting,
    journal: &'a Journal,
}

impl PostingEntry<'_> {
    fn accn(&self) -> AccnEntry<'_> {
        self.data().accn.into_accn(&self.journal.accns)
    }

    fn data(&self) -> &PostingData {
        &self.journal.txns.postings[&self.posting]
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
