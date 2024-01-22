use pest::iterators::Pairs;
use pest_derive::Parser;

use crate::{
    accn::{AccnEntry, AccnEntryMut, AccnTree},
    valuable::{CurrencyStore, Money, MoneyBuilder},
};

#[derive(Parser)]
#[grammar = "./parser/coin.pest"]
struct IdentParser;

struct CoinParser {
    currency_store: CurrencyStore,
    accn_tree: AccnTree,
}

impl CoinParser {
    fn new() -> Self {
        let currency_store = CurrencyStore::new();
        let accn_tree = AccnTree::new();
        Self {
            currency_store,
            accn_tree,
        }
    }

    fn parse_accn(&mut self, pairs: Pairs<Rule>) -> AccnEntryMut {
        dbg!(&pairs);
        pairs.fold(self.accn_tree.root_mut(), |accn, pair| {
            debug_assert_eq!(pair.as_rule(), Rule::ident);
            accn.or_open_child(pair.as_str())
        })
    }

    fn parse_money(&mut self, mut pairs: Pairs<Rule>) -> Option<Money> {
        let mut builder = MoneyBuilder::default();
        let pairs = pairs.next().unwrap().into_inner();

        for pair in pairs {
            match pair.as_rule() {
                Rule::symbol => builder.with_symbol(pair.as_str()),
                Rule::number => builder.with_amount(pair.as_str().parse().unwrap()),
                Rule::code => builder.with_code(pair.as_str()),
                Rule::neg => builder.neg(),
                _ => unreachable!(),
            };
        }

        builder.into_money(&self.currency_store)
    }
}

#[cfg(test)]
mod test {
    use core::panic;

    use itertools::Itertools;
    use pest::{iterators::Pairs, Parser};

    use super::*;

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
            let m = parse_money(m);
            let m = parser
                .parse_money(m)
                .unwrap_or_else(|| panic!("money parser failed"));

            let m = m.fmt(&parser.currency_store);
            assert_eq!(m, e);
        }
    }

    #[test]
    fn test_accn() {
        let accn = vec!["assets"];
        let mut parser = CoinParser::new();

        for a in accn {
            let pairs = IdentParser::parse(Rule::accn, a).unwrap_or_else(|e| panic!("{}", e));
            let accn = parser.parse_accn(pairs);
            assert_eq!(accn.to_string(), a);
        }
    }

    #[test]
    fn test_ident() {
        let file = "./example/simple.coin";
        let input = std::fs::read_to_string(file).unwrap();
        IdentParser::parse(Rule::grammar, &input).unwrap_or_else(|e| panic!("{}", e));
    }
}
