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
use puzzle::LinePassExt;

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
                let hints = CrowdedCluePass.apply_horz(&mut puzzle);
                //println!("\nAfter crowded clue horz:\n{}", puzzle);
                for hint in hints {
                    println!("{:?}", hint);
                }

                let hints = CrowdedCluePass.apply_vert(&mut puzzle);
                //println!("\nAfter crowded clue vert:\n{}", puzzle);
                for hint in hints {
                    println!("{:?}", hint);
                }

                let mut pass_counter = 1;
                let mut is_dirty = true;
                while is_dirty {
                    is_dirty = false;

                    let hints = ContinuousRangePass.apply_horz(&mut puzzle);
                    is_dirty = is_dirty || !hints.is_empty();
                    //println!("\nAfter continuous range horz:\n{}", puzzle);
                    for hint in hints {
                        println!("{:?}", hint);
                    }

                    let hints = ContinuousRangePass.apply_vert(&mut puzzle);
                    is_dirty = is_dirty || !hints.is_empty();
                    for hint in hints {
                        println!("{:?}", hint);
                    }
                    //println!("\nAfter continuous range vert:\n{}", puzzle);
                    pass_counter += 1;
                }
                println!("Number of passes: {}", pass_counter - 1);
                println!("{}", &puzzle);
                println!("{}", puzzle.into_ast());
            }
            Err(e) => panic!("{}", e),
        }
    }
}
