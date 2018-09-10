extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::iterators::Pair;
use pest::Parser;

#[derive(Parser)]
#[grammar = "nono.pest"]
struct NonoParser;

struct Clue(Vec<usize>);

impl<'a> From<Pair<'a, Rule>> for Clue {
    fn from(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::clue);
        Clue(
            pair.into_inner()
                .map(|n| usize::from_str_radix(n.as_str(), 10).unwrap())
                .collect::<Vec<_>>(),
        )
    }
}

fn main() {
    let clue_pairs = NonoParser::parse(Rule::clue, "1 2 3").unwrap_or_else(|e| panic!("{}", e));
    for clue_pair in clue_pairs {
        let clue = Clue::from(clue_pair.clone());
        println!("{:?}", clue.0);
    }
}
