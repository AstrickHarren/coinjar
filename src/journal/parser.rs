use std::io::Write;

use anyhow::{Context, Ok, Result};
use chrono::NaiveDate;

use pest::{
    iterators::{Pair, Pairs},
    Parser, Span,
};
use pest_derive::Parser;

use crate::{
    accn::{AccnEntryMut, AccnTree},
    journal::{Journal, Txn, TxnBuilder, TxnStore},
    valuable::{CurrencyStore, Money, MoneyBuilder, MoneyEntry},
};

#[derive(Parser)]
#[grammar = "./parser/coin.pest"]
pub(crate) struct IdentParser;

fn parse_err(msg: &str, span: Span) -> pest::error::Error<Rule> {
    use pest::error::{Error, ErrorVariant};
    Error::new_from_span(
        ErrorVariant::CustomError {
            message: msg.to_string(),
        },
        span,
    )
}

struct CoinParser {
    currency_store: CurrencyStore,
    accn_tree: AccnTree,
    txn_store: TxnStore,
}

impl CoinParser {
    fn new() -> Self {
        let currency_store = CurrencyStore::new();
        let accn_tree = AccnTree::new();
        let txn_store = TxnStore::default();
        Self {
            currency_store,
            accn_tree,
            txn_store,
        }
    }

    fn parse_accn(&mut self, pair: Pair<Rule>) -> AccnEntryMut {
        let pairs = pair.into_inner();
        pairs.fold(self.accn_tree.root_mut(), |accn, pair| {
            debug_assert_eq!(pair.as_rule(), Rule::ident);
            accn.or_open_child(pair.as_str())
        })
    }

    fn parse_money_builder(pair: Pair<Rule>) -> Result<MoneyBuilder> {
        let pairs = pair.into_inner();
        let mut builder = MoneyBuilder::default();

        for pair in pairs {
            match pair.as_rule() {
                Rule::symbol => builder.with_symbol(pair.as_str()),
                Rule::number => builder.with_amount(pair.as_str().parse().unwrap()),
                Rule::code => builder.with_code(pair.as_str()),
                Rule::neg => builder.neg(),
                _ => unreachable!(),
            };
        }

        Ok(builder)
    }

    fn parse_money(&mut self, pair: Pair<Rule>) -> Result<Money> {
        let builder = Self::parse_money_builder(pair)?;
        builder.into_money(&self.currency_store)
    }

    fn parse_txn(&mut self, pair: Pair<Rule>, date: NaiveDate) -> Result<Txn> {
        let span = pair.as_span();

        let mut pairs = pair.into_inner();
        let desc = pairs.next().unwrap().as_str().to_string();
        let mut txn = TxnBuilder::new(date, desc);

        for posting in pairs {
            let mut pairs = posting.into_inner();
            let accn = self.parse_accn(pairs.next().unwrap()).as_ref().id();
            let money = pairs
                .next()
                .map(|p| {
                    self.parse_money(Pair::clone(&p))
                        .with_context(|| parse_err("error parsing money", p.as_span()))
                })
                .transpose()?;
            txn.with_posting(accn, money);
        }

        txn.build(&mut self.txn_store)
            .with_context(|| parse_err("error parsing transaction", span))
    }

    fn parse_chapter(&mut self, pair: Pair<Rule>) -> Result<()> {
        let mut pairs = pair.into_inner();
        let date = pairs.next().unwrap().as_str().parse()?;
        for pair in pairs {
            self.parse_txn(pair, date)?;
        }
        Ok(())
    }

    fn parse_journal(mut self, pair: Pairs<Rule>) -> Result<Journal> {
        for pair in pair {
            match pair.as_rule() {
                Rule::chapter => self.parse_chapter(pair)?,
                _ => unreachable!(),
            }
        }

        self.into_journal()
    }

    fn into_journal(self) -> Result<Journal> {
        Ok(Journal::new(
            self.accn_tree,
            self.txn_store,
            self.currency_store,
        ))
    }
}

impl Journal {
    fn from_str(s: &str) -> Result<Self> {
        let parser = CoinParser::new();
        let pairs = IdentParser::parse(Rule::grammar, s)?;

        parser.parse_journal(pairs)
    }

    pub(crate) fn from_file(f: &str) -> Result<Self> {
        let input = std::fs::read_to_string(f)?;
        Self::from_str(&input)
    }

    pub(crate) fn save_to_file(&self, f: &str) -> Result<()> {
        let mut file = std::fs::File::create(f)?;
        file.write_all(self.to_string().as_bytes())?;
        Ok(())
    }

    pub(crate) fn parse_money(&self, money: &str) -> Result<MoneyEntry> {
        let pair = IdentParser::parse(Rule::money, money)?.next().unwrap();
        let money = CoinParser::parse_money_builder(pair)?.into_money(&self.currencies)?;
        Ok(money.into_money(&self.currencies))
    }
}

#[cfg(test)]
mod test {
    use core::panic;
    use std::str::FromStr;

    use pest::{iterators::Pairs, Parser};

    use super::*;

    #[rustfmt::skip]
const TXN_INPUT: &str = 
r#"Opening Balances
    assets:cash:checking  $1000.00
    equity:opening-balances"#;

    #[rustfmt::skip]
const JOURNAL_INPUT: &str = 
r#"2021-01-01 Opening Balances
    assets:cash:checking  $1000.00
    equity:opening-balances"#;

    #[rustfmt::skip]
const JOURNAL_OUTPUT: &str =
r#"2021-01-01 Opening Balances
    assets:cash:checking                                          $1000.00
    equity:opening-balances                                      -$1000.00"#;

    fn parse_money(money: &str) -> Pairs<Rule> {
        IdentParser::parse(Rule::money_test, money).unwrap_or_else(|e| panic!("{}", e))
    }

    #[test]
    fn test_money() {
        let money = vec![
            ("$10", "$10"),
            ("-$10", "-$10"),
            ("10£", "10£"),
            ("-10£", "-10£"),
            ("10 GBP", "10£"),
            ("-10 GBP", "-10£"),
            ("$10.00", "$10.00"),
            ("$-10.00", "-$10.00"),
            ("10.00£", "10.00£"),
            ("-10.00£", "-10.00£"),
            ("10.00 GBP", "10.00£"),
            ("-10.00 GBP", "-10.00£"),
        ];

        let mut parser = CoinParser::new();
        for (m, e) in money {
            let mut m = parse_money(m);
            let m = parser
                .parse_money(m.next().unwrap())
                .unwrap_or_else(|e| panic!("money parser failed: {}", e));

            let m = m.fmt(&parser.currency_store);
            assert_eq!(m, e);
        }
    }

    #[test]
    fn test_accn() {
        let accn = vec!["assets"];
        let mut parser = CoinParser::new();

        for a in accn {
            let mut pairs = IdentParser::parse(Rule::accn, a).unwrap_or_else(|e| panic!("{}", e));
            let accn = parser.parse_accn(pairs.next().unwrap());
            assert_eq!(accn.to_string(), a);
        }
    }

    #[test]
    fn test_txn() {
        let mut parser = CoinParser::new();
        let mut pairs =
            IdentParser::parse(Rule::booking, TXN_INPUT).unwrap_or_else(|e| panic!("{}", e));
        let txn = parser
            .parse_txn(
                pairs.next().unwrap(),
                NaiveDate::from_str("2021-01-01").unwrap(),
            )
            .unwrap_or_else(|e| panic!("{:#}", e));
        let journal = parser.into_journal().unwrap_or_else(|e| panic!("{}", e));
        assert_eq!(txn.into_txn(&journal).to_string(), JOURNAL_OUTPUT);
    }

    #[test]
    fn test_ident() -> Result<()> {
        let parser = CoinParser::new();
        let pairs =
            IdentParser::parse(Rule::grammar, JOURNAL_INPUT).unwrap_or_else(|e| panic!("{:#}", e));
        let journal = parser
            .parse_journal(pairs)
            .unwrap_or_else(|e| panic!("{:#}", e));
        assert_eq!(journal.to_string(), JOURNAL_OUTPUT);
        Ok(())
    }
}
