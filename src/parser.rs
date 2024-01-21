use pest::iterators::Pairs;
use pest_derive::Parser;

use crate::valuable::{CurrencyStore, Money, MoneyBuilder};

#[derive(Parser)]
#[grammar = "./parser/coin.pest"]
struct IdentParser;

struct CoinParser {
    currency_store: CurrencyStore,
}

impl CoinParser {
    fn parse_money(&mut self, mut pairs: Pairs<Rule>) -> Option<Money> {
        dbg!(&pairs);
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

        dbg!(&builder);
        builder.into_money(&self.currency_store)
    }
}

#[cfg(test)]
mod test {
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

        let mut parser = CoinParser {
            currency_store: CurrencyStore::new(),
        };

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
    fn test_ident() {
        let file = "./example/simple.coin";
        let input = std::fs::read_to_string(file).unwrap();
        IdentParser::parse(Rule::grammar, &input).unwrap_or_else(|e| panic!("{}", e));
    }
}
