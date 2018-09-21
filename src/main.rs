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
                let mut pass_counter = 0;

                let hints = CrowdedCluePass.apply_horz(&mut puzzle);
                pass_counter += 1;
                println!("\nCrowded clue horz ({}):", pass_counter);
                for hint in hints {
                    println!("{:?}", hint);
                }
                println!("{}", puzzle);

                let hints = CrowdedCluePass.apply_vert(&mut puzzle);
                pass_counter += 1;
                println!("\nCrowded clue vert ({}):", pass_counter);
                for hint in hints {
                    println!("{:?}", hint);
                }
                println!("{}", puzzle);

                let mut is_dirty = true;
                while is_dirty {
                    is_dirty = false;

                    let hints = ContinuousRangePass.apply_horz(&mut puzzle);
                    is_dirty = is_dirty || !hints.is_empty();
                    pass_counter += 1;
                    println!("\nContinuous range horz ({}):", pass_counter);
                    for hint in hints {
                        println!("{:?}", hint);
                    }
                    println!("{}", puzzle);

                    let hints = ContinuousRangePass.apply_vert(&mut puzzle);
                    is_dirty = is_dirty || !hints.is_empty();
                    pass_counter += 1;
                    println!("\nContinuous range vert ({}):", pass_counter);
                    for hint in hints {
                        println!("{:?}", hint);
                    }
                    println!("{}", puzzle);
                }
                println!("Number of passes: {}", pass_counter - 1);
                println!("{}", &puzzle);
                println!("{}", puzzle.into_ast());
            }
            Err(e) => panic!("{}", e),
        }
    }
}
