extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate fixedbitset;
#[macro_use]
extern crate itertools;

mod parser;
mod pass;
mod puzzle;

use pest::Parser;
use std::io;
use std::io::BufRead;

use parser::NonoParser;
use parser::Rule;
use pass::ContinuousRangePass;
use pass::CrowdedCluePass;
use pass::DiscreteRangePass;
use puzzle::LinePass;
use puzzle::LinePassExt;
use puzzle::Orientation;
use puzzle::Puzzle;

fn apply<T: LinePass>(
    pass: &T,
    orientation: &Orientation,
    puzzle: &mut Puzzle,
    pass_num: usize,
) -> bool {
    let hints = pass.apply(orientation, puzzle);
    let is_dirty = !hints.is_empty();
    println!("\n{:?} {:?} ({}):", pass, orientation, pass_num);
    for hint in hints {
        println!("{:?}", hint);
    }
    println!("{}", puzzle);
    is_dirty
}

fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let ast = NonoParser::parse(Rule::puzzle, &line)
            .unwrap_or_else(|e| panic!("{}", e))
            .next()
            .map(parser::Puzzle::from)
            .unwrap();
        match puzzle::Puzzle::try_from_ast(ast) {
            Ok(mut puzzle) => {
                let mut pass_counter = 0;

                for orientation in Orientation::iter() {
                    pass_counter += 1;
                    apply(&CrowdedCluePass, &orientation, &mut puzzle, pass_counter);
                }

                let mut is_dirty = true;
                while is_dirty {
                    is_dirty = false;

                    for orientation in Orientation::iter() {
                        pass_counter += 1;
                        if apply(
                            &ContinuousRangePass,
                            &orientation,
                            &mut puzzle,
                            pass_counter,
                        ) {
                            is_dirty = true;
                        }

                        if !is_dirty
                            && apply(&DiscreteRangePass, &orientation, &mut puzzle, pass_counter)
                        {
                            is_dirty = true;
                        }
                    }
                }
                println!("{}", puzzle.into_ast());
            }
            Err(e) => panic!("{}", e),
        }
    }
}
