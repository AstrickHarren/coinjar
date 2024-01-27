pub mod entry;
pub mod parser;
pub mod register;

use std::{collections::HashMap, fmt::Display};

use anyhow::{anyhow, Result};
use chrono::NaiveDate;

use colored::Colorize;
use itertools::Itertools;
use rust_decimal::prelude::Zero;
use uuid::Uuid;

use crate::{
    accn::{Accn, AccnTree},
    valuable::{CurrencyStore, Money, Valuable},
};

use self::entry::{PostingEntry, TxnEntry};

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

    fn with_strict_posting_combined(&mut self, accn: Accn, money: Money) -> &mut Self {
        match self
            .postings
            .iter_mut()
            .find(|p| p.accn == accn && p.money.eq_currency(&money))
        {
            Some(posting) => {
                posting.money += money;
                self
            }
            None => self.with_strict_posting(accn, money),
        }
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

    pub(crate) fn with_posting_combined(&mut self, accn: Accn, money: Option<Money>) -> &mut Self {
        match money {
            Some(money) => self.with_strict_posting_combined(accn, money),
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

pub(crate) struct TxnBuilderMut<'a> {
    builder: TxnBuilder,
    journal: &'a mut Journal,
}

impl<'a> TxnBuilderMut<'a> {
    pub(crate) fn with_posting(
        mut self,
        accn: impl Into<Accn>,
        money: Option<impl Into<Money>>,
    ) -> Self {
        self.builder
            .with_posting(accn.into(), money.map(|m| m.into()));
        self
    }

    pub(crate) fn with_posting_combined(
        mut self,
        accn: impl Into<Accn>,
        money: Option<impl Into<Money>>,
    ) -> Self {
        self.builder
            .with_posting_combined(accn.into(), money.map(|m| m.into()));
        self
    }

    pub(crate) fn build(self) -> Result<TxnEntry<'a>> {
        let txn = self.builder.build(&mut self.journal.txns)?;
        Ok(TxnEntry::new(txn, self.journal))
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

    pub(crate) fn txn(&self, txn: Txn) -> TxnEntry<'_> {
        TxnEntry::new(txn, self)
    }

    pub(crate) fn postings(&self) -> impl Iterator<Item = PostingEntry<'_>> {
        self.txns
            .postings
            .keys()
            .copied()
            .map(move |posting| posting.into_posting(self))
    }

    pub(crate) fn new_txn(&mut self, date: NaiveDate, desc: String) -> TxnBuilderMut<'_> {
        TxnBuilderMut {
            builder: TxnBuilder::new(date, desc),
            journal: self,
        }
    }

    pub(crate) fn accns(&self) -> &AccnTree {
        &self.accns
    }

    pub(crate) fn accns_mut(&mut self) -> &mut AccnTree {
        &mut self.accns
    }
}

impl Display for Journal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            writeln!(f, "{}", "Accns:".cyan().bold())?;
            self.accns.fmt(f)?;

            writeln!(f, "\n{}", "Transactions:".cyan().bold())?;
        }
        self.txns().format("\n\n").fmt(f)
    }
}
