use pest::iterators::Pair;
use pest_derive::Parser;

use crate::accn::{AccnMut, AccnStore};

struct Booking {
    desc: String,
}

#[derive(Debug, Parser)]
#[grammar = "../share/grammar.pest"]
struct CoinParser;

impl AccnStore {
    fn parse_accn(&mut self, pair: Pair<'_, Rule>) -> AccnMut<'_> {
        let mut pairs = pair.into_inner();
        let mut accn = self
            .find_accn_mut(pairs.next().unwrap().as_str())
            .unwrap()
            .id();

        while let Some(pair) = pairs.next() {
            let name = pair.as_str();
            accn = self
                .find_accn_mut(name)
                .map(|accn| accn.id())
                .unwrap_or_else(|| self.accn_mut(accn).open_child_accn(name).id());
        }

        self.accn_mut(accn)
    }
}

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
