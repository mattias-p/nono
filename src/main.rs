extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::iterators::Pair;
use pest::Parser;
use std::fmt;

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

impl fmt::Display for Clue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some((first, rest)) = self.0.split_first() {
            write!(f, "{}", first)?;
            for number in rest {
                write!(f, " {}", number)?;
            }
        }
        Ok(())
    }
}

struct ClueList(Vec<Clue>);

impl<'a> From<Pair<'a, Rule>> for ClueList {
    fn from(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::clue_list);
        ClueList(pair.into_inner().map(Clue::from).collect())
    }
}

impl fmt::Display for ClueList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (first, rest) = self.0.split_first().unwrap();
        write!(f, "{}", first)?;
        for number in rest {
            write!(f, ",{}", number)?;
        }
        Ok(())
    }
}

fn main() {
    let pairs = NonoParser::parse(Rule::clue_list, "1 2 3,4 5").unwrap_or_else(|e| panic!("{}", e));
    for pair in pairs {
        let clue = ClueList::from(pair.clone());
        println!("{}", clue);
    }
}
