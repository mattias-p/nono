use fixedbitset::FixedBitSet;
use parser::Cell;

use puzzle::Line;
use puzzle::LineHint;
use puzzle::LineMut;
use puzzle::LinePass;

#[derive(Debug, Eq)]
pub struct FilledRun {
    start: usize,
    end: usize,
    numbers: FixedBitSet,
}

impl LineHint for FilledRun {
    fn check(&self, line: &Line) -> bool {
        line.range_contains_unfilled(self.start..self.end)
    }
    fn apply(&self, line: &mut LineMut) {
        line.fill_range(self.start..self.end)
    }
}

impl PartialEq for FilledRun {
    fn eq(&self, other: &Self) -> bool {
        self.start == other.start
            && self.end == other.end
            && self.numbers.intersection(&other.numbers).count()
                == self.numbers.count_ones(0..self.numbers.len())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct CrossedRun {
    start: usize,
    end: usize,
}

impl LineHint for CrossedRun {
    fn check(&self, line: &Line) -> bool {
        line.range_contains_uncrossed(self.start..self.end)
    }
    fn apply(&self, line: &mut LineMut) {
        line.cross_range(self.start..self.end)
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum DiscreteRangeHint {
    CrossedRun(CrossedRun),
    FilledRun(FilledRun),
}

impl LineHint for DiscreteRangeHint {
    fn check(&self, line: &Line) -> bool {
        match self {
            DiscreteRangeHint::CrossedRun(inner) => inner.check(line),
            DiscreteRangeHint::FilledRun(inner) => inner.check(line),
        }
    }
    fn apply(&self, line: &mut LineMut) {
        match self {
            DiscreteRangeHint::CrossedRun(inner) => inner.apply(line),
            DiscreteRangeHint::FilledRun(inner) => inner.apply(line),
        }
    }
}

#[derive(Clone, Copy)]
enum State {
    Empty(usize),
    Filled(usize, usize),
    End,
}

impl State {
    fn start() -> State {
        State::Empty(0)
    }
    fn cell(self, cell: Cell) -> Self {
        match (self, cell) {
            (State::Empty(_), Cell::Crossed) => State::Empty(0),
            (State::Empty(n), Cell::Undecided) => State::Empty(n + 1),
            (State::Empty(n), Cell::Filled) => State::Filled(1, n + 1),
            (State::Filled(m, n), Cell::Undecided) => State::Filled(m + 1, n + 1),
            (State::Filled(m, n), Cell::Filled) => State::Filled(m + 1, n + 1),
            (State::Filled(_, _), Cell::Crossed) => State::End,
            (State::End, _) => State::End,
            (_, Cell::Impossible) => State::End,
        }
    }
}

struct Iter<'a> {
    line: &'a Line,
    number: usize,
    focus: usize,
    state: State,
}

impl<'a> Iter<'a> {
    fn new(line: &'a Line, number: usize, start: usize) -> Self {
        Iter {
            line,
            number,
            focus: start,
            state: State::start(),
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        for focus in self.focus..self.line.len() {
            self.state = match self.state.cell(self.line.get(focus)) {
                State::Filled(m, _) if m > self.number => State::End,
                state => state,
            };
            let emit = match self.state {
                State::Filled(_, n) if n >= self.number => true,
                State::Empty(n) if n >= self.number => true,
                _ => false,
            };
            if emit && (focus + 1 >= self.line.len() || !self.line.is_filled(focus + 1)) {
                self.focus = focus + 1;
                return Some(self.focus - self.number);
            }
        }
        self.focus = self.line.len();
        None
    }
}

struct Possibilities {
    filled: FixedBitSet,
    crossed: FixedBitSet,
    cell_numbers: FixedBitSet,
}

impl Possibilities {
    fn new(line_len: usize, clue_len: usize) -> Self {
        let cell_numbers = FixedBitSet::with_capacity(line_len * clue_len);
        let mut filled = FixedBitSet::with_capacity(line_len);
        let mut crossed = FixedBitSet::with_capacity(line_len);

        filled.set_range(0..line_len, true);
        crossed.set_range(0..line_len, true);

        Possibilities {
            filled,
            crossed,
            cell_numbers,
        }
    }

    fn positions(&mut self, positions: &[usize], clue: &[usize]) {
        //println!(" OK {} {:?}", start, &positions);
        let mut old_end = 0;
        for ((number_index, number), start) in clue.iter().enumerate().zip(positions) {
            //println!("  filled {}..{}", old_end, start);
            for j in old_end..*start {
                self.filled.set(j, false);
            }
            //println!("  crossed {}..{}", *start, *start + number);
            for j in *start..*start + number {
                self.crossed.set(j, false);
                self.cell_numbers.put(j * clue.len() + number_index);
            }
            old_end = *start + number;
        }
        //println!("  filled {}..{}", old_end, line.len());
        for j in old_end..self.filled.len() {
            self.filled.set(j, false);
        }
    }

    fn solve(
        &mut self,
        line: &Line,
        clue: &[usize],
        depth: usize,
        start: usize,
        positions: &mut Vec<usize>,
    ) {
        if let Some(number) = clue.get(depth) {
            for start in Iter::new(line, *number, start) {
                positions.push(start);
                self.solve(line, clue, depth + 1, start + number + 1, positions);
                positions.pop();
            }
        } else if !line.range_contains_filled(start..line.len()) {
            self.positions(positions, clue);
        }
    }

    fn hints(&self, line: &Line, clue: &[usize]) -> Vec<Box<DiscreteRangeHint>> {
        /*
        println!("filled {:?}", self.filled.ones().collect::<Vec<_>>());
        println!("crossed {:?}", self.crossed.ones().collect::<Vec<_>>());
        println!(
            "cell_numbers {:?}",
            self.cell_numbers.ones().collect::<Vec<_>>()
        );
        */

        let mut hints: Vec<Box<DiscreteRangeHint>> = vec![];
        let mut i = 0;
        while i < self.filled.len() {
            while i < self.filled.len() && !self.filled.contains(i) && !self.crossed.contains(i) {
                i += 1;
            }
            if i >= self.crossed.len() {
                break;
            }
            let start = i;
            if self.filled.contains(i) {
                while i < line.len() && self.filled.contains(i) {
                    i += 1;
                }
                let mut numbers = FixedBitSet::with_capacity(clue.len());
                for j in 0..clue.len() {
                    if self.cell_numbers.contains(start * clue.len() + j) {
                        numbers.put(j);
                    }
                }
                let filled_run = FilledRun {
                    start,
                    end: i,
                    numbers,
                };
                if filled_run.check(line) {
                    hints.push(Box::new(DiscreteRangeHint::FilledRun(filled_run)));
                }
            } else {
                while i < line.len() && self.crossed.contains(i) {
                    i += 1;
                }
                let crossed_run = CrossedRun { start, end: i };
                if crossed_run.check(line) {
                    hints.push(Box::new(DiscreteRangeHint::CrossedRun(crossed_run)));
                }
            }
        }
        //for h in &hints { println!("{:?}", h); }
        hints
    }
}

#[derive(Debug)]
pub struct DiscreteRangePass;

impl LinePass for DiscreteRangePass {
    type Hint = DiscreteRangeHint;

    fn run(&self, clue: &[usize], line: &Line) -> Vec<Box<Self::Hint>> {
        let mut possibilities = Possibilities::new(line.len(), clue.len());

        possibilities.solve(line, clue, 0, 0, &mut vec![]);

        possibilities.hints(line, clue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use puzzle::Grid;
    use std::iter::FromIterator;

    #[test]
    fn run1() {
        let mut grid = Grid::new(4, 1);
        let mut line = grid.horz_mut(0);
        line.fill(0);
        line.fill(2);
        let hints = DiscreteRangePass.run(&[1, 1], &line);
        assert_eq!(
            hints,
            vec![
                Box::new(DiscreteRangeHint::CrossedRun(CrossedRun {
                    start: 1,
                    end: 2,
                })),
                Box::new(DiscreteRangeHint::CrossedRun(CrossedRun {
                    start: 3,
                    end: 4,
                })),
            ]
        );
    }

    #[test]
    fn run2() {
        let mut grid = Grid::new(4, 1);
        let mut line = grid.horz_mut(0);
        line.fill(2);
        let hints = DiscreteRangePass.run(&[2], &line);
        assert_eq!(
            hints,
            vec![Box::new(DiscreteRangeHint::CrossedRun(CrossedRun {
                start: 0,
                end: 1,
            }))]
        );
    }

    #[test]
    fn run3() {
        let mut grid = Grid::new(6, 1);
        let mut line = grid.horz_mut(0);
        line.fill(0);
        line.cross(2);
        line.fill(4);
        line.cross(5);
        let hints = DiscreteRangePass.run(&[1, 2], &line);
        assert_eq!(
            hints,
            vec![
                Box::new(DiscreteRangeHint::CrossedRun(CrossedRun {
                    start: 1,
                    end: 3,
                })),
                Box::new(DiscreteRangeHint::FilledRun(FilledRun {
                    start: 3,
                    end: 5,
                    numbers: FixedBitSet::from_iter(vec![1]),
                })),
            ]
        );
    }

    #[test]
    fn run4() {
        let mut grid = Grid::new(4, 1);
        let mut line = grid.horz_mut(0);
        line.cross(1);
        line.fill(2);
        line.cross(3);
        let hints = DiscreteRangePass.run(&[1], &line);
        assert_eq!(
            hints,
            vec![Box::new(DiscreteRangeHint::CrossedRun(CrossedRun {
                start: 0,
                end: 2,
            }))]
        );
    }

    #[test]
    fn run5() {
        let mut grid = Grid::new(4, 1);
        let mut line = grid.horz_mut(0);
        line.cross(0);
        line.fill(1);
        line.cross(2);
        let hints = DiscreteRangePass.run(&[1], &line);
        assert_eq!(
            hints,
            vec![Box::new(DiscreteRangeHint::CrossedRun(CrossedRun {
                start: 2,
                end: 4,
            }))]
        );
    }

    #[test]
    fn run6() {
        let mut grid = Grid::new(7, 1);
        let mut line = grid.horz_mut(0);
        line.fill(1);
        line.cross(2);
        let hints = DiscreteRangePass.run(&[2, 1], &line);
        assert_eq!(
            hints,
            vec![Box::new(DiscreteRangeHint::FilledRun(FilledRun {
                start: 0,
                end: 2,
                numbers: FixedBitSet::from_iter(vec![0]),
            }))]
        );
    }

    #[test]
    fn run7() {
        let mut grid = Grid::new(7, 1);
        let mut line = grid.horz_mut(0);
        line.cross(2);
        line.fill(3);
        line.cross(4);
        line.fill_range(5..7);
        let hints = DiscreteRangePass.run(&[1, 2], &line);
        assert_eq!(
            hints,
            vec![Box::new(DiscreteRangeHint::CrossedRun(CrossedRun {
                start: 0,
                end: 3,
            }))]
        );
    }
}
