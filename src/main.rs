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
use pass::ContinuousRangeHint;
use pass::ContinuousRangePass;
use pass::CrowdedClue;
use pass::CrowdedCluePass;
use pass::DiscreteRangeHint;
use pass::DiscreteRangePass;
use pest::Parser;
use puzzle::LineMut;
use puzzle::LinePassExt;
use puzzle::Orientation;
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

#[derive(Debug)]
enum Hint {
    CrowdedClue(CrowdedClue),
    ContinuousRange(ContinuousRangeHint),
    DiscreteRange(DiscreteRangeHint),
}

use puzzle::Line;

impl puzzle::LineHint for Hint {
    fn check(&self, line: &Line) -> bool {
        match self {
            Hint::CrowdedClue(inner) => inner.check(line),
            Hint::ContinuousRange(inner) => inner.check(line),
            Hint::DiscreteRange(inner) => inner.check(line),
        }
    }
    fn apply(&self, line: &mut LineMut) {
        match self {
            Hint::CrowdedClue(inner) => inner.apply(line),
            Hint::ContinuousRange(inner) => inner.apply(line),
            Hint::DiscreteRange(inner) => inner.apply(line),
        }
    }
}

#[derive(Debug)]
enum Pass {
    CrowdedClue(CrowdedCluePass),
    ContinuousRange(ContinuousRangePass),
    DiscreteRange(DiscreteRangePass),
}

impl puzzle::LinePass for Pass {
    type Hint = Hint;
    fn run(&self, clue: &[usize], line: &Line) -> Vec<Box<Self::Hint>> {
        match self {
            Pass::CrowdedClue(inner) => inner
                .run(clue, line)
                .into_iter()
                .map(|hint| Box::new(Hint::CrowdedClue(*hint)))
                .collect(),
            Pass::ContinuousRange(inner) => inner
                .run(clue, line)
                .into_iter()
                .map(|hint| Box::new(Hint::ContinuousRange(*hint)))
                .collect(),
            Pass::DiscreteRange(inner) => inner
                .run(clue, line)
                .into_iter()
                .map(|hint| Box::new(Hint::DiscreteRange(*hint)))
                .collect(),
        }
    }
}

struct Solver<'a> {
    cur_p: usize,
    cur_o: usize,
    fail_count: usize,
    passes: &'a [Pass],
}

impl<'a> Solver<'a> {
    fn new(passes: &'a [Pass]) -> Self {
        Solver {
            cur_p: 0,
            cur_o: 0,
            fail_count: 0,
            passes,
        }
    }

    fn initial(&mut self) -> (&'a Pass, Orientation) {
        let pass = self.passes.get(self.cur_p).unwrap();
        let orientations = Orientation::all();
        let orientation = &orientations[self.cur_o];
        (pass, *orientation)
    }

    fn succeeded(&mut self) -> Option<(&'a Pass, Orientation)> {
        self.fail_count = 0;

        let last_p = self.cur_p;
        if self.cur_p > 1 {
            self.cur_p = 1;
            self.next(last_p)
        } else {
            self.next(last_p)
        }
    }

    fn failed(&mut self) -> Option<(&'a Pass, Orientation)> {
        self.fail_count += 1;

        let last_p = self.cur_p;
        self.next(last_p)
    }

    fn next(&mut self, last_p: usize) -> Option<(&'a Pass, Orientation)> {
        if self.fail_count >= 2 {
            self.cur_p += 1;
            self.fail_count = 0;
        }

        self.cur_o = 1 - self.cur_o;
        if self.cur_o == 0 {
            if let Some(Pass::CrowdedClue(_)) = self.passes.get(last_p) {
                self.cur_p = 1;
            }
        }

        if let Some(pass) = self.passes.get(self.cur_p) {
            let orientations = Orientation::all();
            let orientation = &orientations[self.cur_o];
            return Some((pass, *orientation));
        } else {
            None
        }
    }
}

fn main() {
    let opt = Opt::from_args();

    let stdin = io::stdin();
    let passes: [Pass; 3] = [
        Pass::CrowdedClue(CrowdedCluePass),
        Pass::ContinuousRange(ContinuousRangePass),
        Pass::DiscreteRange(DiscreteRangePass),
    ];
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let ast = NonoParser::parse(Rule::puzzle, &line)
            .unwrap_or_else(|e| panic!("{}", e))
            .next()
            .map(parser::Puzzle::from)
            .unwrap();
        match puzzle::Puzzle::try_from_ast(ast) {
            Ok(mut puzzle) => {
                let mut solver = Solver::new(&passes);

                println!("{}", opt.theme.view(&puzzle));

                let mut next_pass = Some(solver.initial());
                let mut pass_counter = 0;
                while let Some((pass, orientation)) = next_pass {
                    if puzzle.is_complete() {
                        break;
                    }

                    pass_counter += 1;
                    let hints = pass.run_puzzle(&orientation, &puzzle);
                    for hint in &hints {
                        hint.apply(&mut puzzle);
                    }

                    if opt.theme != Theme::Brief {
                        println!("{:?} {:?} ({})", pass, orientation, pass_counter);
                        for hint in &hints {
                            println!("{:?}", hint);
                        }
                    }
                    if !hints.is_empty() {
                        println!("{}", opt.theme.view(&puzzle));
                    }

                    next_pass = if hints.is_empty() {
                        solver.failed()
                    } else {
                        solver.succeeded()
                    };
                }
            }
            Err(e) => panic!("{}", e),
        }
    }
}
