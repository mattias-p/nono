extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate fixedbitset;

mod parser;
mod puzzle;

use parser::NonoParser;
use parser::Rule;
use pest::Parser;
use std::io;
use std::io::BufRead;
use std::iter;

use puzzle::Line;
use puzzle::LinePass;
use puzzle::LinePassExt;

struct OnlyCluesPass;

impl LinePass for OnlyCluesPass {
    fn run(&self, clue: &[usize], line: &mut Line) {
        let sum: usize = clue.iter().sum();
        let freedom: usize = line.len() - (sum + clue.len() - 1);
        let mut x0 = 0;
        for number in clue.iter() {
            if *number > freedom {
                line.fill_range(x0 + freedom..x0 + number);
            }
            x0 += number + 1;
        }
    }
}

struct ContinuousRangePass;

impl LinePass for ContinuousRangePass {
    fn run(&self, clue: &[usize], line: &mut Line) {
        let w = line.len();
        //println!("{}", &puzzle);
        //println!("CLUE  {}", line.clue);

        let mut range_starts = Vec::with_capacity(clue.len());
        let mut x0 = 0;
        for number in clue.iter() {
            let mut x = x0;
            while x < w && x < x0 + number {
                if line.is_crossed(x) {
                    // pushing cross
                    x0 = x + 1;
                }
                x += 1;
            }
            if x < w && line.is_filled(x) {
                // pulling fill
                while x < w && line.is_filled(x) {
                    x += 1;
                }
                // TODO check for impossibility
                x0 = x - number;
            }
            assert!(x0 <= w - number);
            range_starts.push(x0);
            x0 += number;
        }

        let mut range_ends = Vec::with_capacity(clue.len());
        let mut x1 = w + 1;
        for number in clue.iter().rev() {
            x1 -= 1;
            let mut x = x1 - 1;
            while x > 0 && x > x1 - number {
                if line.is_crossed(x) {
                    // pushing cross
                    x1 = x;
                }
                x -= 1;
            }

            if x > 0 && line.is_filled(x - 1) {
                // pulling fill
                while x > 0 && line.is_filled(x - 1) {
                    x -= 1;
                }
                // TODO check for impossibility
                x1 = x + number;
            }
            assert!(x1 <= w);
            assert!(x1 >= *number);
            range_ends.push(x1);
            x1 -= number;
        }

        line.cross_range(0..range_starts[0]);
        line.cross_range(range_ends[0]..w);

        for ((((number, range_start), range_end), prev_range_end), next_min_start) in clue
            .iter()
            .zip(range_starts.iter())
            .zip(range_ends.iter().rev())
            .zip(iter::once(&0).chain(range_ends.iter().rev()))
            .zip(range_starts.iter().skip(1).chain(iter::once(&w)))
        {
            assert!(range_start + number <= *range_end);

            let turf_start = *prev_range_end.max(range_start);
            let turf_end = *range_end.min(next_min_start);

            //println!("number {}", number);
            //println!("range  {}..{}", range_start, range_end);
            //println!("turf   {}..{}", turf_start, turf_end);

            if *range_end == *range_start + number {
                if *range_start > 0 {
                    line.cross(range_start - 1);
                }
                line.fill_range(*range_start..*range_end);
                if *range_end < w {
                    line.cross(*range_end);
                }
                continue;
            }

            if *range_start + 2 * number > *range_end {
                let kernel_start = range_end - number;
                let kernel_end = range_start + number;

                //println!("kernel {}..{}", kernel_start, kernel_end);

                line.fill_range(kernel_start..kernel_end);

                if let Some(x0) = (turf_start..kernel_start).find(|x| line.is_filled(*x)) {
                    line.fill_range(x0..kernel_start);
                    line.cross_range(x0 + number..turf_end);
                }

                if let Some(x1) = (kernel_end..turf_end).rev().find(|x| line.is_filled(*x)) {
                    line.fill_range(kernel_end..x1);
                    line.cross_range(turf_start..x1 - number);
                }
            } else {
                //println!("{} {} - - {} {}", range_start, turf_start, turf_end, range_end);
                if let Some(x0) = (turf_start..turf_end).find(|x| line.is_filled(*x)) {
                    //println!("x0 {}", x0);
                    line.cross_range(x0 + number..turf_end);
                    line.cross_range(turf_start..(x0 + 1).max(*number) - number);

                    if let Some(x1) = (x0..turf_end).rev().find(|x| line.is_filled(*x)) {
                        line.fill_range(x0 + 1..x1);
                        line.cross_range(turf_start..x1.max(*number) - number);
                    } else {
                        line.cross_range(turf_start..x0.max(*number) - number);
                    }
                }
            }
        }
    }
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
                for _x in 0..1 {
                    ContinuousRangePass.apply(&mut puzzle);
                    println!("{}", puzzle);
                }
                println!("{}", puzzle.into_ast());
            }
            Err(e) => panic!("{}", e),
        }
    }
}
