extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate fixedbitset;
#[macro_use]
extern crate itertools;
#[macro_use]
extern crate structopt;

mod parser;
mod pass;
mod puzzle;

use std::io;
use std::io::BufRead;

use parser::NonoParser;
use parser::Rule;
use pass::ContinuousRangePass;
use pass::CrowdedCluePass;
use pass::DiscreteRangePass;
use pest::Parser;
use puzzle::LinePass;
use puzzle::LinePassExt;
use puzzle::Orientation;
use puzzle::Puzzle;
use puzzle::Theme;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "nono")]
/// A nonogram hint dispenser
///
/// Available display themes: ascii, unicode, brief
struct Opt {
    /// Select display theme
    #[structopt(short = "t", long = "theme", default_value = "unicode")]
    theme: Theme,
}

fn apply<T: LinePass>(
    puzzle: &mut Puzzle,
    pass: &T,
    orientation: &Orientation,
    theme: &Theme,
    pass_num: usize,
) -> bool {
    let hints = pass.apply(orientation, puzzle);
    let is_dirty = !hints.is_empty();
    if *theme != Theme::Brief {
        println!("\n{:?} {:?} ({}):", pass, orientation, pass_num);
        for hint in hints {
            println!("{:?}", hint);
        }
    }
    println!("{}", theme.view(puzzle));
    is_dirty
}

fn main() {
    let opt = Opt::from_args();

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
                    apply(
                        &mut puzzle,
                        &CrowdedCluePass,
                        &orientation,
                        &opt.theme,
                        pass_counter,
                    );
                }

                let mut is_dirty = true;
                while is_dirty {
                    is_dirty = false;

                    for orientation in Orientation::iter() {
                        pass_counter += 1;
                        if apply(
                            &mut puzzle,
                            &ContinuousRangePass,
                            &orientation,
                            &opt.theme,
                            pass_counter,
                        ) {
                            is_dirty = true;
                        }

                        if !is_dirty && apply(
                            &mut puzzle,
                            &DiscreteRangePass,
                            &orientation,
                            &opt.theme,
                            pass_counter,
                        ) {
                            is_dirty = true;
                        }
                    }
                }
            }
            Err(e) => panic!("{}", e),
        }
    }
}
