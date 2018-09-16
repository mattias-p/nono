use std::iter;

use puzzle::Line;
use puzzle::LineHint;
use puzzle::LinePass;

pub trait ClueExt {
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

#[derive(Debug)]
enum ContinuousRangeHint {
    Unreachable {
        reachable_start: usize,
        reachable_end: usize,
    },
    Kernel {
        kernel_start: usize,
        kernel_end: usize,
    },
    Termination {
        range_start: usize,
        range_end: usize,
    },
    TurfNearSingleton {
        found_start: usize,
        kernel_start: usize,
        reachable_end: usize,
        turf_end: usize,
    },
    TurfFarSingleton {
        turf_start: usize,
        reachable_start: usize,
        kernel_end: usize,
        found_end: usize,
    },
    TurfPair {
        turf_start: usize,
        reachable_start: usize,
        found_start: usize,
        found_end: usize,
        reachable_end: usize,
        turf_end: usize,
    },
    TurfSingleton {
        turf_start: usize,
        reachable_start: usize,
        reachable_end: usize,
        turf_end: usize,
    },
}

impl LineHint for ContinuousRangeHint {
    fn check(&self, line: &Line) -> bool {
        match self {
            &ContinuousRangeHint::Unreachable {
                reachable_start,
                reachable_end,
            } => {
                let len = line.len();
                line.range_contains_uncrossed(0..reachable_start)
                    || line.range_contains_uncrossed(reachable_end..len)
            }
            &ContinuousRangeHint::Kernel {
                kernel_start,
                kernel_end,
            } => line.range_contains_unfilled(kernel_start..kernel_end),
            &ContinuousRangeHint::Termination {
                range_start,
                range_end,
            } => {
                (range_start > 0 && !line.is_crossed(range_start - 1))
                    || (range_end < line.len() && !line.is_crossed(range_end))
            }
            &ContinuousRangeHint::TurfNearSingleton {
                found_start,
                kernel_start,
                reachable_end,
                turf_end,
            } => {
                line.range_contains_unfilled(found_start..kernel_start)
                    || line.range_contains_uncrossed(reachable_end..turf_end)
            }
            &ContinuousRangeHint::TurfFarSingleton {
                turf_start,
                reachable_start,
                kernel_end,
                found_end,
            } => {
                line.range_contains_uncrossed(turf_start..reachable_start)
                    || line.range_contains_unfilled(kernel_end..found_end)
            }
            &ContinuousRangeHint::TurfPair {
                turf_start,
                reachable_start,
                found_start,
                found_end,
                reachable_end,
                turf_end,
            } => {
                line.range_contains_uncrossed(turf_start..reachable_start)
                    || line.range_contains_unfilled(found_start + 1..found_end - 1)
                    || line.range_contains_uncrossed(reachable_end..turf_end)
            }
            &ContinuousRangeHint::TurfSingleton {
                turf_start,
                reachable_start,
                reachable_end,
                turf_end,
            } => {
                line.range_contains_uncrossed(turf_start..reachable_start)
                    || line.range_contains_uncrossed(reachable_end..turf_end)
            }
        }
    }
    fn apply(&self, line: &mut Line) {
        match self {
            &ContinuousRangeHint::Unreachable {
                reachable_start,
                reachable_end,
            } => {
                let len = line.len();
                line.cross_range(0..reachable_start);
                line.cross_range(reachable_end..len);
            }
            &ContinuousRangeHint::Kernel {
                kernel_start,
                kernel_end,
            } => {
                line.fill_range(kernel_start..kernel_end);
            }
            &ContinuousRangeHint::Termination {
                range_start,
                range_end,
            } => {
                if range_start > 0 {
                    line.cross(range_start - 1);
                }
                if range_end < line.len() {
                    line.cross(range_end);
                }
            }
            &ContinuousRangeHint::TurfNearSingleton {
                found_start,
                kernel_start,
                reachable_end,
                turf_end,
            } => {
                line.fill_range(found_start..kernel_start);
                line.cross_range(reachable_end..turf_end);
            }
            &ContinuousRangeHint::TurfFarSingleton {
                turf_start,
                reachable_start,
                kernel_end,
                found_end,
            } => {
                line.cross_range(turf_start..reachable_start);
                line.fill_range(kernel_end..found_end);
            }
            &ContinuousRangeHint::TurfPair {
                turf_start,
                reachable_start,
                found_start,
                found_end,
                reachable_end,
                turf_end,
            } => {
                line.cross_range(turf_start..reachable_start);
                line.fill_range(found_start + 1..found_end - 1);
                line.cross_range(reachable_end..turf_end);
            }
            &ContinuousRangeHint::TurfSingleton {
                turf_start,
                reachable_start,
                reachable_end,
                turf_end,
            } => {
                line.cross_range(turf_start..reachable_start);
                line.cross_range(reachable_end..turf_end);
            }
        }
    }
}

pub struct ContinuousRangePass;

impl LinePass for ContinuousRangePass {
    fn run(&self, clue: &[usize], line: &Line) -> Vec<Box<LineHint>> {
        let mut hints: Vec<Box<LineHint>> = vec![];
        //println!("CLUE  {:?}", clue);

        let range_starts = clue.range_starts(line);
        let range_ends = clue.range_ends(line);

        // unreachable cells
        let unreachable = ContinuousRangeHint::Unreachable {
            reachable_start: range_starts[0],
            reachable_end: range_ends[0],
        };
        if unreachable.check(line) {
            hints.push(Box::new(unreachable));
        }

        let len = line.len();
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

            if range_start + 2 * number > range_end {
                let kernel_start = range_end - number;
                let kernel_end = range_start + number;
                //println!("kernel {}..{}", kernel_start, kernel_end);

                // kernel
                let kernel = ContinuousRangeHint::Kernel {
                    kernel_start,
                    kernel_end,
                };
                if kernel.check(line) {
                    hints.push(Box::new(kernel));
                }

                if kernel_start == range_start && kernel_end == range_end {
                    let termination = ContinuousRangeHint::Termination {
                        range_start,
                        range_end,
                    };
                    if termination.check(line) {
                        hints.push(Box::new(termination));
                    }
                    continue;
                }

                // kernel turf
                if let Some(found_start) = (turf_start..kernel_start).find(|x| line.is_filled(*x)) {
                    let turf_near_singleton = ContinuousRangeHint::TurfNearSingleton {
                        found_start,
                        kernel_start,
                        reachable_end: found_start + number,
                        turf_end,
                    };
                    if turf_near_singleton.check(line) {
                        hints.push(Box::new(turf_near_singleton));
                    }
                }
                if let Some(found_end) = (kernel_end..turf_end).rev().find(|x| line.is_filled(*x)) {
                    let turf_far_singleton = ContinuousRangeHint::TurfFarSingleton {
                        turf_start,
                        reachable_start: found_end - number,
                        kernel_end,
                        found_end,
                    };
                    if turf_far_singleton.check(line) {
                        hints.push(Box::new(turf_far_singleton));
                    }
                }
            } else if let Some(found_start) = (turf_start..turf_end).find(|x| line.is_filled(*x)) {
                let reachable_end = found_start + number;
                if let Some(found_end) = (found_start + 1..turf_end)
                    .rev()
                    .find(|x| line.is_filled(*x))
                {
                    let turf_pair = ContinuousRangeHint::TurfPair {
                        turf_start,
                        reachable_start: found_end.saturating_sub(number),
                        found_start,
                        found_end,
                        reachable_end,
                        turf_end,
                    };
                    if turf_pair.check(line) {
                        hints.push(Box::new(turf_pair));
                    }
                } else {
                    let turf_singleton = ContinuousRangeHint::TurfSingleton {
                        turf_start,
                        reachable_start: found_start.saturating_sub(number),
                        reachable_end,
                        turf_end,
                    };
                    if turf_singleton.check(line) {
                        hints.push(Box::new(turf_singleton));
                    }
                }
            }
        }
        hints
    }
}
