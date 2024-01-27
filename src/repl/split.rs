use anyhow::{anyhow, bail};
use split::util::find_or_create_accn;

use crate::{accn::Accn, journal::entry::TxnEntry, valuable::Money};

use super::*;

enum SplitSt {
    Money,
    Accn,
    Desc,
    Payee,
}
impl SplitSt {
    fn keywords() -> &'static [&'static str] {
        &["on", "for", "by"]
    }
}
impl FromStr for SplitSt {
    type Err = String;
    fn from_str(s: &str) -> std::prelude::v1::Result<Self, Self::Err> {
        match s {
            "on" => Ok(SplitSt::Accn),
            "for" => Ok(SplitSt::Desc),
            "by" => Ok(SplitSt::Payee),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Default)]
struct SplitBuilder {
    money: Option<Money>,
    desc: Option<String>,
    recv: Option<Accn>,
    payees: Vec<Accn>,
}

impl SplitBuilder {
    fn with_money(&mut self, money: impl Into<Money>) -> &mut Self {
        self.money = Some(money.into());
        self
    }

    fn with_recv(&mut self, recv: impl Into<Accn>) -> &mut Self {
        self.recv = Some(recv.into());
        self
    }

    fn with_desc(&mut self, desc: impl Into<String>) -> &mut Self {
        self.desc = Some(desc.into());
        self
    }

    fn with_payee(&mut self, payee: impl Into<Accn>) -> &mut Self {
        self.payees.push(payee.into());
        self
    }

    fn build(mut self, journal: &mut Journal, date: NaiveDate) -> Result<TxnEntry> {
        let money = self.money.ok_or_else(|| anyhow!("missing money"))?;
        let recv = self.recv.ok_or_else(|| anyhow!("missing recv"))?;
        let desc = self.desc.unwrap_or_default();
        if self.payees.is_empty() {
            bail!("missing payees");
        }

        let moneys = money.split(self.payees.len(), 2);
        let mut txn = journal.new_txn(date, desc).with_posting(recv, Some(-money));

        for money in moneys {
            txn = txn.with_posting_combined(self.payees.pop().unwrap(), Some(money));
        }
        txn.build()
    }
}

/// Split a transaction, args must have the format:
/// `<money> (on <accn>) (for <desc>) (by <payee> <payee> ...)`
pub(super) fn split<'a>(
    journal: &'a mut Journal,
    args: &[String],
    state: &ReplState,
) -> Result<TxnEntry<'a>> {
    let mut st = SplitSt::Money;
    let mut builder = SplitBuilder::default();

    for arg in args {
        if SplitSt::keywords().contains(&arg.as_str()) {
            st = arg.parse().unwrap();
            continue;
        }

        match st {
            SplitSt::Money => {
                let money = journal.parse_money(arg)?;
                builder.with_money(money);
            }
            SplitSt::Accn => {
                builder.with_recv(find_or_create_accn(journal, arg)?);
            }
            SplitSt::Desc => {
                builder.with_desc(arg);
            }
            SplitSt::Payee => {
                builder.with_payee(find_or_create_accn(journal, arg)?);
            }
        }
    }

    builder.build(journal, state.date)
}
