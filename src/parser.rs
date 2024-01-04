use pest_derive::Parser;

struct Booking {
    desc: String,
}

#[derive(Debug, Parser)]
#[grammar = "../share/grammar.pest"]
struct CoinParser;

#[cfg(test)]
mod test {

    use pest::Parser;

    use super::*;

    #[test]
    fn parse_example() {
        let file = std::fs::read_to_string("journal.coin").unwrap();
        let pairs = CoinParser::parse(Rule::grammar, &file).unwrap_or_else(|e| panic!("{}", e));
        dbg!(pairs);
    }
}
