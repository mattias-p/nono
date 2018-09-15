extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate fixedbitset;
#[macro_use]
extern crate itertools;

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

trait ClueExt {
    fn range_starts(&self, line: &Line) -> Vec<usize>;
    fn range_ends(&self, line: &Line) -> Vec<usize>;
}

impl<'a> ClueExt for &'a [usize] {
    fn range_starts(&self, line: &Line) -> Vec<usize> {
        let mut range_starts = Vec::with_capacity(self.len());
        let mut start = 0;
        for number in self.iter() {
            //println!("  starts start {}", start);
            //println!("  starts number {}", number);
            let mut focus = start;
            while focus < (start + number).min(line.len()) {
                if line.is_crossed(focus) {
                    // pushing cross
                    //println!("  starts pushed by cross at {}", focus);
                    start = focus + 1;
                }
                focus += 1;
            }
            while focus < line.len() && line.is_filled(focus) {
                // pulling fill
                //println!("  starts pulled by fill at {}", focus);
                focus += 1;
            }
            range_starts.push(focus - number);
            start = focus + 1;
        }
        //println!("  starts {:?}", range_starts);
        range_starts
    }

    fn range_ends(&self, line: &Line) -> Vec<usize> {
        let mut range_ends = Vec::with_capacity(self.len());
        let mut last: isize = line.len() as isize - 1;
        for number in self.iter().rev() {
            let number = *number as isize;
            //println!("  ends last {}", last);
            //println!("  ends number {}", number);
            let mut focus: isize = last;
            while focus >= 0 && focus + number >= last + 1 {
                if line.is_crossed(focus as usize) {
                    // pushing cross
                    //println!("  ends pushed by cross at {}", focus);
                    last = focus - 1;
                }
                focus -= 1;
            }
            while focus >= 0 && line.is_filled(focus as usize) {
                // pulling fill
                //println!("  ends pulled by fill at {}", focus);
                focus -= 1;
            }
            assert!(focus + number + 1 <= line.len() as isize);
            range_ends.push((focus + 1 + number) as usize);
            last = focus - 1;
        }
        //println!("  ends {:?}", range_ends);
        range_ends
    }
}

struct ContinuousRangePass;

impl LinePass for ContinuousRangePass {
    fn run(&self, clue: &[usize], line: &mut Line) {
        //println!("CLUE  {:?}", clue);

        let range_starts = clue.range_starts(line);
        let range_ends = clue.range_ends(line);

        // unreachable cells
        let len = line.len();
        line.cross_range(0..range_starts[0]);
        line.cross_range(range_ends[0]..len);
        if line.check_dirty() {
            println!("unreachable cells");
        }

        let turf_ends = range_starts
            .iter()
            .skip(1)
            .map(|e| *e - 1)
            .chain(iter::once(len + 1))
            .zip(range_ends.iter().rev())
            .map(|(next_range_start, range_end)| (*range_end).min(next_range_start));
        let turf_starts = iter::once(0)
            .chain(range_ends.iter().rev().map(|e| e + 1))
            .zip(range_starts.iter())
            .map(|(prev_range_end, range_start)| prev_range_end.max(*range_start));
        let range_ends = range_ends.iter().rev().map(|e| *e);
        let range_starts = range_starts.iter().map(|e| *e);
        let numbers = clue.iter().map(|e| *e);

        for (number, range_start, range_end, turf_start, turf_end) in
            izip!(numbers, range_starts, range_ends, turf_starts, turf_ends)
        {
            //println!("number {}", number);
            //println!("range  {}..{}", range_start, range_end);
            //println!("turf   {}..{}", turf_start, turf_end);

            if range_start + number == range_end {
                // perfect fit
                if range_start > 0 {
                    line.cross(range_start - 1);
                }
                line.fill_range(range_start..range_end);
                if range_end < line.len() {
                    line.cross(range_end);
                }
                if line.check_dirty() {
                    println!("perfect fit");
                }
                continue;
            }

            if range_start + 2 * number > range_end {
                let kernel_start = range_end - number;
                let kernel_end = range_start + number;
                //println!("kernel {}..{}", kernel_start, kernel_end);

                // kernel
                line.fill_range(kernel_start..kernel_end);
                if line.check_dirty() {
                    println!("kernel");
                }

                // kernel turf
                if let Some(found_start) = (turf_start..kernel_start).find(|x| line.is_filled(*x)) {
                    line.fill_range(found_start..kernel_start);
                    line.cross_range(found_start + number..turf_end);
                }
                if let Some(found_end) = (kernel_end..turf_end).rev().find(|x| line.is_filled(*x)) {
                    line.fill_range(kernel_end..found_end);
                    line.cross_range(turf_start..found_end - number);
                }
                if line.check_dirty() {
                    println!("kernel turf");
                }
            } else if let Some(found_start) = (turf_start..turf_end).find(|x| line.is_filled(*x)) {
                // drifting turf
                line.cross_range(turf_start..found_start.saturating_sub(number));
                if let Some(found_end) = (found_start..turf_end).rev().find(|x| line.is_filled(*x))
                {
                    line.fill_range(found_start + 1..found_end);
                    line.cross_range(found_end + number..turf_end);
                } else {
                    line.cross_range(found_start + number..turf_end);
                }
                if line.check_dirty() {
                    println!("drifting turf");
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
                for _x in 0..20 {
                    ContinuousRangePass.apply_horz(&mut puzzle);
                    //println!("\nAfter horz:\n{}", puzzle);
                    ContinuousRangePass.apply_vert(&mut puzzle);
                    //println!("\nAfter vert:\n{}", puzzle);
                }
                println!("{}", &puzzle);
                println!("{}", puzzle.into_ast());
            }
            Err(e) => panic!("{}", e),
        }
    }
}
