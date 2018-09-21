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
use puzzle::LinePass;
use puzzle::LinePassExt;
use puzzle::Puzzle;

fn horz<T: LinePass>(pass: &T, puzzle: &mut Puzzle, pass_num: usize) -> bool {
    let hints = pass.apply_horz(puzzle);
    let is_dirty = !hints.is_empty();
    println!("\n{:?} horz ({}):", pass, pass_num);
    for hint in hints {
        println!("{:?}", hint);
    }
    println!("{}", puzzle);
    is_dirty
}

fn vert<T: LinePass>(pass: &T, puzzle: &mut Puzzle, pass_num: usize) -> bool {
    let hints = pass.apply_vert(puzzle);
    let is_dirty = !hints.is_empty();
    println!("\n{:?} vert ({}):", pass, pass_num);
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

                pass_counter += 1;
                horz(&CrowdedCluePass, &mut puzzle, pass_counter);

                pass_counter += 1;
                vert(&CrowdedCluePass, &mut puzzle, pass_counter);

                let mut is_dirty = true;
                while is_dirty {
                    is_dirty = false;

                    pass_counter += 1;
                    if horz(&ContinuousRangePass, &mut puzzle, pass_counter) {
                        is_dirty = true;
                    }

                    pass_counter += 1;
                    if vert(&ContinuousRangePass, &mut puzzle, pass_counter) {
                        is_dirty = true;
                    }
                }
                println!("{}", puzzle.into_ast());
            }
            Err(e) => panic!("{}", e),
        }
    }
}
