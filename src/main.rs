extern crate pest;
#[macro_use]
extern crate pest_derive;

mod parser;

use parser::NonoParser;
use parser::Puzzle;
use parser::Rule;
use pest::Parser;
use std::io;
use std::io::BufRead;

fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let pair = NonoParser::parse(Rule::puzzle, &line)
            .unwrap_or_else(|e| panic!("{}", e))
            .next()
            .unwrap();
        println!("{}", Puzzle::from(pair));
    }
}
