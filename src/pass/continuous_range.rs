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
            start = line.bump_start(start, *number);
            range_starts.push(start);
            start += number + 1;
        }
        //println!("  starts {:?}", range_starts);
        range_starts
    }

    fn range_ends(&self, line: &Line) -> Vec<usize> {
        let mut range_ends = Vec::with_capacity(self.len());
        let mut last = line.len() as isize - 1;
        for number in self.iter().rev() {
            last = line.bump_last(last as usize, *number);
            range_ends.push(last as usize);
            last -= *number as isize + 2;
        }
        //println!("  ends {:?}", range_ends);
        range_ends
    }
}

#[derive(Debug)]
pub struct Unreachable {
    reachable_start: usize,
    reachable_end: usize,
}

impl LineHint for Unreachable {
    fn check(&self, line: &Line) -> bool {
        let len = line.len();
        line.range_contains_uncrossed(0..self.reachable_start)
            || line.range_contains_uncrossed(self.reachable_end..len)
    }
    fn apply(&self, line: &mut Line) {
        let len = line.len();
        line.cross_range(0..self.reachable_start);
        line.cross_range(self.reachable_end..len);
    }
}

#[derive(Debug)]
pub struct Kernel {
    kernel_start: usize,
    kernel_end: usize,
}

impl LineHint for Kernel {
    fn check(&self, line: &Line) -> bool {
        line.range_contains_unfilled(self.kernel_start..self.kernel_end)
    }
    fn apply(&self, line: &mut Line) {
        line.fill_range(self.kernel_start..self.kernel_end);
    }
}

#[derive(Debug)]
pub struct Termination {
    range_start: usize,
    range_end: usize,
}

impl LineHint for Termination {
    fn check(&self, line: &Line) -> bool {
        (self.range_start > 0 && !line.is_crossed(self.range_start - 1))
            || (self.range_end < line.len() && !line.is_crossed(self.range_end))
    }
    fn apply(&self, line: &mut Line) {
        if self.range_start > 0 {
            line.cross(self.range_start - 1);
        }
        if self.range_end < line.len() {
            line.cross(self.range_end);
        }
    }
}

#[derive(Debug)]
pub struct TurfNearSingleton {
    found_start: usize,
    kernel_start: usize,
    reachable_end: usize,
    turf_end: usize,
}

impl LineHint for TurfNearSingleton {
    fn check(&self, line: &Line) -> bool {
        line.range_contains_unfilled(self.found_start..self.kernel_start)
            || line.range_contains_uncrossed(self.reachable_end..self.turf_end)
    }
    fn apply(&self, line: &mut Line) {
        line.fill_range(self.found_start..self.kernel_start);
        line.cross_range(self.reachable_end..self.turf_end);
    }
}

#[derive(Debug)]
pub struct TurfFarSingleton {
    turf_start: usize,
    reachable_start: usize,
    kernel_end: usize,
    found_end: usize,
}

impl LineHint for TurfFarSingleton {
    fn check(&self, line: &Line) -> bool {
        line.range_contains_uncrossed(self.turf_start..self.reachable_start)
            || line.range_contains_unfilled(self.kernel_end..self.found_end)
    }
    fn apply(&self, line: &mut Line) {
        line.cross_range(self.turf_start..self.reachable_start);
        line.fill_range(self.kernel_end..self.found_end);
    }
}

#[derive(Debug)]
pub struct TurfPair {
    turf_start: usize,
    reachable_start: usize,
    found_start: usize,
    found_end: usize,
    reachable_end: usize,
    turf_end: usize,
}

impl LineHint for TurfPair {
    fn check(&self, line: &Line) -> bool {
        line.range_contains_uncrossed(self.turf_start..self.reachable_start)
            || line.range_contains_unfilled(self.found_start + 1..self.found_end - 1)
            || line.range_contains_uncrossed(self.reachable_end..self.turf_end)
    }
    fn apply(&self, line: &mut Line) {
        line.cross_range(self.turf_start..self.reachable_start);
        line.fill_range(self.found_start + 1..self.found_end - 1);
        line.cross_range(self.reachable_end..self.turf_end);
    }
}

#[derive(Debug)]
pub struct TurfSingleton {
    turf_start: usize,
    reachable_start: usize,
    reachable_end: usize,
    turf_end: usize,
}

impl LineHint for TurfSingleton {
    fn check(&self, line: &Line) -> bool {
        line.range_contains_uncrossed(self.turf_start..self.reachable_start)
            || line.range_contains_uncrossed(self.reachable_end..self.turf_end)
    }
    fn apply(&self, line: &mut Line) {
        line.cross_range(self.turf_start..self.reachable_start);
        line.cross_range(self.reachable_end..self.turf_end);
    }
}

#[derive(Debug)]
pub enum ContinuousRangeHint {
    Unreachable(Unreachable),
    Kernel(Kernel),
    Termination(Termination),
    TurfNearSingleton(TurfNearSingleton),
    TurfFarSingleton(TurfFarSingleton),
    TurfPair(TurfPair),
    TurfSingleton(TurfSingleton),
}

impl LineHint for ContinuousRangeHint {
    fn check(&self, line: &Line) -> bool {
        match self {
            ContinuousRangeHint::Unreachable(inner) => inner.check(line),
            ContinuousRangeHint::Kernel(inner) => inner.check(line),
            ContinuousRangeHint::Termination(inner) => inner.check(line),
            ContinuousRangeHint::TurfNearSingleton(inner) => inner.check(line),
            ContinuousRangeHint::TurfFarSingleton(inner) => inner.check(line),
            ContinuousRangeHint::TurfPair(inner) => inner.check(line),
            ContinuousRangeHint::TurfSingleton(inner) => inner.check(line),
        }
    }
    fn apply(&self, line: &mut Line) {
        match self {
            ContinuousRangeHint::Unreachable(inner) => inner.apply(line),
            ContinuousRangeHint::Kernel(inner) => inner.apply(line),
            ContinuousRangeHint::Termination(inner) => inner.apply(line),
            ContinuousRangeHint::TurfNearSingleton(inner) => inner.apply(line),
            ContinuousRangeHint::TurfFarSingleton(inner) => inner.apply(line),
            ContinuousRangeHint::TurfPair(inner) => inner.apply(line),
            ContinuousRangeHint::TurfSingleton(inner) => inner.apply(line),
        }
    }
}

pub struct ContinuousRangePass;

impl LinePass for ContinuousRangePass {
    type Hint = ContinuousRangeHint;
    fn run(&self, clue: &[usize], line: &Line) -> Vec<Box<Self::Hint>> {
        let mut hints: Vec<Box<Self::Hint>> = vec![];
        //println!("CLUE  {:?}", clue);

        let range_starts = clue.range_starts(line);
        let range_ends = clue.range_ends(line);

        // unreachable cells
        let unreachable = Unreachable {
            reachable_start: range_starts[0],
            reachable_end: range_ends[0],
        };
        if unreachable.check(line) {
            hints.push(Box::new(ContinuousRangeHint::Unreachable(unreachable)));
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
                let kernel = Kernel {
                    kernel_start,
                    kernel_end,
                };
                if kernel.check(line) {
                    hints.push(Box::new(ContinuousRangeHint::Kernel(kernel)));
                }

                if kernel_start == range_start && kernel_end == range_end {
                    let termination = Termination {
                        range_start,
                        range_end,
                    };
                    if termination.check(line) {
                        hints.push(Box::new(ContinuousRangeHint::Termination(termination)));
                    }
                    continue;
                }

                // kernel turf
                if let Some(found_start) = (turf_start..kernel_start).find(|x| line.is_filled(*x)) {
                    let turf_near_singleton = TurfNearSingleton {
                        found_start,
                        kernel_start,
                        reachable_end: found_start + number,
                        turf_end,
                    };
                    if turf_near_singleton.check(line) {
                        hints.push(Box::new(ContinuousRangeHint::TurfNearSingleton(
                            turf_near_singleton,
                        )));
                    }
                }
                if let Some(found_end) = (kernel_end..turf_end).rev().find(|x| line.is_filled(*x)) {
                    let turf_far_singleton = TurfFarSingleton {
                        turf_start,
                        reachable_start: found_end - number,
                        kernel_end,
                        found_end,
                    };
                    if turf_far_singleton.check(line) {
                        hints.push(Box::new(ContinuousRangeHint::TurfFarSingleton(
                            turf_far_singleton,
                        )));
                    }
                }
            } else if let Some(found_start) = (turf_start..turf_end).find(|x| line.is_filled(*x)) {
                let reachable_end = found_start + number;
                if let Some(found_end) = (found_start + 1..turf_end)
                    .rev()
                    .find(|x| line.is_filled(*x))
                {
                    let turf_pair = TurfPair {
                        turf_start,
                        reachable_start: found_end.saturating_sub(number),
                        found_start,
                        found_end,
                        reachable_end,
                        turf_end,
                    };
                    if turf_pair.check(line) {
                        hints.push(Box::new(ContinuousRangeHint::TurfPair(turf_pair)));
                    }
                } else {
                    let turf_singleton = TurfSingleton {
                        turf_start,
                        reachable_start: found_start.saturating_sub(number),
                        reachable_end,
                        turf_end,
                    };
                    if turf_singleton.check(line) {
                        hints.push(Box::new(ContinuousRangeHint::TurfSingleton(turf_singleton)));
                    }
                }
            }
        }
        hints
    }
}
