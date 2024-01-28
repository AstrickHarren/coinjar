use anyhow::{anyhow, bail};

use inquire::Text;
use pest::{iterators::Pairs, Parser};
use split::util::find_or_create_accn;

use crate::{
    accn::Accn,
    journal::{
        entry::TxnEntry,
        parser::{IdentParser, Rule},
    },
    valuable::Money,
};

use super::*;

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
        let desc = self.desc.ok_or(()).or_else(|_| {
            Text::new("enter desc")
                .prompt()
                .map_err(|e| anyhow!("{}", e))
        })?;
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

    fn from_str(journal: &mut Journal, input: &str) -> Result<Self> {
        let pairs = IdentParser::parse(Rule::split, input)?;
        Self::from_pairs(journal, pairs)
    }

    fn from_pairs(journal: &mut Journal, mut pairs: Pairs<Rule>) -> Result<Self> {
        let mut builder = Self::default();

        let money = journal.parse_money(pairs.next().unwrap().as_str())?;
        builder.with_money(money);

        for pair in pairs {
            match pair.as_rule() {
                Rule::from_accn => {
                    let accn = pair
                        .into_inner()
                        .exactly_one()
                        .map_err(|e| anyhow!("{}", e))?;
                    builder.with_recv(find_or_create_accn(journal, accn.as_str())?);
                }
                Rule::to_accn => {
                    for pair in pair.into_inner() {
                        builder.with_payee(find_or_create_accn(journal, pair.as_str())?);
                    }
                }
                Rule::desc => {
                    builder.with_desc(pair.as_str());
                }
                _ => unreachable!("unexpected rule: {:?}", pair.as_rule()),
            }
        }

        Ok(builder)
    }
}

pub(super) fn split<'a>(
    journal: &'a mut Journal,
    pairs: Pairs<'_, Rule>,
    state: &ReplState,
) -> Result<TxnEntry<'a>> {
    SplitBuilder::from_pairs(journal, pairs)?.build(journal, state.date)
}

#[cfg(test)]
mod test {
    use pest::Parser;

    use crate::journal::parser::{IdentParser, Rule};

    #[test]
    fn test_parse_split() {
        let cmd = "split 100 usd from food to groceries    , snacks ";
        let pairs = IdentParser::parse(Rule::split, cmd).unwrap_or_else(|e| panic!("{}", e));
        dbg!(pairs);
    }
}
