pub mod entry;

use std::{collections::HashMap, fmt::Display};

use anyhow::{anyhow, Result};
use chrono::NaiveDate;

use itertools::Itertools;
use rust_decimal::prelude::Zero;
use uuid::Uuid;

use crate::{
    accn::{Accn, AccnTree},
    valuable::{CurrencyStore, Money, Valuable},
};

use self::entry::TxnEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Posting {
    id: Uuid,
}

impl Posting {
    fn new() -> Self {
        Self { id: Uuid::new_v4() }
    }
}

#[derive(Debug)]
struct PostingData {
    accn: Accn,
    money: Money,
    txn: Txn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct Txn {
    id: Uuid,
}

#[derive(Debug)]
struct TxnData {
    date: NaiveDate,
    description: String,
    postings: Vec<Posting>,
}

#[derive(Default, Debug)]
pub(crate) struct TxnStore {
    txns: HashMap<Txn, TxnData>,
    postings: HashMap<Posting, PostingData>,
}

pub(crate) struct TxnBuilder {
    date: NaiveDate,
    desc: String,
    postings: Vec<PostingData>,
    inferred_posting: Option<Accn>,

    txn: Txn,
}

impl TxnBuilder {
    pub(crate) fn new(date: NaiveDate, desc: String) -> Self {
        Self {
            date,
            desc,
            postings: Vec::new(),
            txn: Txn { id: Uuid::new_v4() },
            inferred_posting: None,
        }
    }

    fn with_strict_posting(&mut self, accn: Accn, money: Money) -> &mut Self {
        self.postings.push(PostingData {
            accn,
            money,
            txn: self.txn,
        });
        self
    }

    fn with_inferred_posting(&mut self, accn: Accn) -> &mut Self {
        self.inferred_posting = Some(accn);
        self
    }

    fn inbalance(&self) -> Valuable {
        self.postings.iter().map(|posting| posting.money).sum()
    }

    pub(crate) fn with_posting(&mut self, accn: Accn, money: Option<Money>) -> &mut Self {
        match money {
            Some(money) => self.with_strict_posting(accn, money),
            None => self.with_inferred_posting(accn),
        }
    }

    fn try_infer_inbalence(&mut self) -> Result<()> {
        let inbalance = self.inbalance();

        match !inbalance.is_zero() {
            true => {
                for money in inbalance {
                    self.with_strict_posting(
                        self.inferred_posting
                            .ok_or_else(|| anyhow!("transaction not balanced"))?,
                        -money,
                    );
                }
            }
            false => (),
        };

        Ok(())
    }

    pub(crate) fn build(mut self, txn_store: &mut TxnStore) -> Result<Txn> {
        self.try_infer_inbalence()?;

        let (posting_id, posting): (Vec<_>, Vec<_>) = self
            .postings
            .into_iter()
            .map(|p| (Posting::new(), p))
            .unzip();

        let txn = TxnData {
            date: self.date,
            description: self.desc,
            postings: posting_id.clone(),
        };

        txn_store.txns.insert(self.txn, txn);
        txn_store
            .postings
            .extend(posting_id.into_iter().zip(posting));

        Ok(self.txn)
    }
}

#[derive(Debug)]
pub(crate) struct Journal {
    accns: AccnTree,
    txns: TxnStore,
    currencies: CurrencyStore,
}

impl Journal {
    pub(crate) fn new(accns: AccnTree, txns: TxnStore, currencies: CurrencyStore) -> Self {
        Self {
            accns,
            txns,
            currencies,
        }
    }

    pub(crate) fn txns(&self) -> impl Iterator<Item = TxnEntry<'_>> {
        self.txns
            .txns
            .keys()
            .copied()
            .map(move |txn| TxnEntry::new(txn, self))
    }
}

impl Display for Journal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.txns().format("\n\n").fmt(f)
    }
}
