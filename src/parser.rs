use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "./parser/coin.pest"]
struct IdentParser;

#[cfg(test)]
mod test {
    use pest::{iterators::Pairs, Parser};

    use super::*;

    fn parse_money(money: &str) -> Pairs<Rule> {
        IdentParser::parse(Rule::money, money).unwrap_or_else(|e| panic!("{}", e))
    }

    #[test]
    fn test_money() {
        let money = vec![
            "$10",
            "-$10",
            "10£",
            "-10£",
            "10 GBP",
            "-10 GBP",
            "$10.00",
            "$-10.00",
            "10.00£",
            "-10.00£",
            "10.00 GBP",
            "-10.00 GBP",
        ];

        for m in money {
            parse_money(m);
        }
    }

    #[test]
    fn test_ident() {
        let file = "./example/simple.coin";
        let input = std::fs::read_to_string(file).unwrap();
        IdentParser::parse(Rule::grammar, &input).unwrap_or_else(|e| panic!("{}", e));
    }
}
