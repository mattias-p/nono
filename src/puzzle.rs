use fixedbitset::FixedBitSet;
use parser::Cell;
use parser::ClueList;
use parser::GridLine;
use std::fmt;
use std::ops::Range;

use parser;

pub trait LineHint: fmt::Debug {
    fn check(&self, line: &Line) -> bool;
    fn apply(&self, line: &mut Line);
}

#[derive(Debug)]
enum Orientation {
    Horz,
    Vert,
}

#[derive(Debug)]
pub struct Hint {
    orientation: Orientation,
    line: usize,
    line_hint: Box<LineHint>,
}

pub trait LinePass {
    fn run(&self, clue: &[usize], line: &Line) -> Vec<Box<LineHint>>;
}

pub trait LinePassExt {
    fn apply_horz(&self, puzzle: &mut Puzzle) -> Vec<Hint>;
    fn apply_vert(&self, puzzle: &mut Puzzle) -> Vec<Hint>;
}

impl<T: LinePass> LinePassExt for T {
    fn apply_horz(&self, puzzle: &mut Puzzle) -> Vec<Hint> {
        let mut hints = vec![];
        for (y, clue) in puzzle.horz_clues.0.iter().enumerate() {
            let mut line = HorzLine {
                grid: &mut puzzle.grid,
                y,
            };
            for line_hint in self.run(clue.0.as_slice(), &line) {
                line_hint.apply(&mut line);
                hints.push(Hint {
                    orientation: Orientation::Horz,
                    line: y,
                    line_hint,
                });
            }
            //println!("\nAfter horz line:\n{}", puzzle);
        }
        hints
    }
    fn apply_vert(&self, puzzle: &mut Puzzle) -> Vec<Hint> {
        let mut hints = vec![];
        for (x, clue) in puzzle.vert_clues.0.iter().enumerate() {
            let mut line = VertLine {
                grid: &mut puzzle.grid,
                x,
            };
            for line_hint in self.run(clue.0.as_slice(), &line) {
                line_hint.apply(&mut line);
                hints.push(Hint {
                    orientation: Orientation::Vert,
                    line: x,
                    line_hint,
                });
            }
            //println!("\nAfter vert line:\n{}", puzzle);
        }
        hints
    }
}

pub trait Line {
    fn len(&self) -> usize;
    fn is_crossed(&self, i: usize) -> bool;
    fn is_filled(&self, i: usize) -> bool;
    fn cross(&mut self, i: usize);
    fn fill(&mut self, i: usize);

    fn cross_range(&mut self, r: Range<usize>) {
        for i in r {
            self.cross(i);
        }
    }

    fn fill_range(&mut self, r: Range<usize>) {
        for i in r {
            self.fill(i);
        }
    }

    fn range_contains_unfilled(&self, r: Range<usize>) -> bool {
        for i in r {
            if !self.is_filled(i) {
                return true;
            }
        }
        false
    }

    fn range_contains_uncrossed(&self, r: Range<usize>) -> bool {
        for i in r {
            if !self.is_crossed(i) {
                return true;
            }
        }
        false
    }

    fn bump_start(&self, mut start: usize, number: usize) -> usize {
        //println!("  starts start {}", start);
        //println!("  starts number {}", number);
        let mut focus = start;
        while focus < (start + number).min(self.len()) {
            if self.is_crossed(focus) {
                // pushing cross
                //println!("  starts pushed by cross at {}", focus);
                start = focus + 1;
            }
            focus += 1;
        }
        while focus < self.len() && self.is_filled(focus) {
            // pulling fill
            //println!("  starts pulled by fill at {}", focus);
            focus += 1;
        }

        focus - number
    }

    fn bump_last(&self, last: usize, number: usize) -> isize {
        let mut last = last as isize;
        let number = number as isize;
        //println!("  ends last {}", last);
        //println!("  ends number {}", number);
        let mut focus: isize = last;
        while focus >= 0 && focus + number >= last + 1 {
            if self.is_crossed(focus as usize) {
                // pushing cross
                //println!("  ends pushed by cross at {}", focus);
                last = focus - 1;
            }
            focus -= 1;
        }
        while focus >= 0 && self.is_filled(focus as usize) {
            // pulling fill
            //println!("  ends pulled by fill at {}", focus);
            focus -= 1;
        }

        assert!(focus + number + 1 <= self.len() as isize);
        focus + number + 1
    }
}

struct HorzLine<'a> {
    grid: &'a mut Grid,
    y: usize,
}

impl<'a> Line for HorzLine<'a> {
    fn is_crossed(&self, x: usize) -> bool {
        self.grid.is_crossed(x, self.y)
    }
    fn is_filled(&self, x: usize) -> bool {
        self.grid.is_filled(x, self.y)
    }
    fn cross(&mut self, x: usize) {
        self.grid.cross(x, self.y);
    }
    fn fill(&mut self, x: usize) {
        self.grid.fill(x, self.y);
    }
    fn len(&self) -> usize {
        self.grid.width
    }
}

struct VertLine<'a> {
    grid: &'a mut Grid,
    x: usize,
}

impl<'a> Line for VertLine<'a> {
    fn is_crossed(&self, y: usize) -> bool {
        self.grid.is_crossed(self.x, y)
    }
    fn is_filled(&self, y: usize) -> bool {
        self.grid.is_filled(self.x, y)
    }
    fn cross(&mut self, y: usize) {
        self.grid.cross(self.x, y);
    }
    fn fill(&mut self, y: usize) {
        self.grid.fill(self.x, y);
    }
    fn len(&self) -> usize {
        self.grid.height
    }
}

struct Grid {
    width: usize,
    height: usize,
    filled: FixedBitSet,
    crossed: FixedBitSet,
}

impl Grid {
    fn index(&self, x: usize, y: usize) -> usize {
        assert!(x < self.width);
        assert!(y < self.height);
        y * self.width + x
    }
    fn get(&self, x: usize, y: usize) -> Cell {
        let i = self.index(x, y);
        match (self.filled.contains(i), self.crossed.contains(i)) {
            (false, false) => Cell::Undecided,
            (false, true) => Cell::Crossed,
            (true, false) => Cell::Filled,
            (true, true) => Cell::Impossible,
        }
    }
    fn fill(&mut self, x: usize, y: usize) -> bool {
        let i = self.index(x, y);
        let old_value = self.filled.contains(i);
        self.filled.put(i);
        !old_value
    }
    fn cross(&mut self, x: usize, y: usize) -> bool {
        let i = self.index(x, y);
        let old_value = self.crossed.contains(i);
        self.crossed.put(i);
        !old_value
    }
    fn is_crossed(&self, x: usize, y: usize) -> bool {
        let i = self.index(x, y);
        self.crossed.contains(i)
    }
    fn is_filled(&self, x: usize, y: usize) -> bool {
        let i = self.index(x, y);
        self.filled.contains(i)
    }
}

pub struct Puzzle {
    vert_clues: ClueList,
    horz_clues: ClueList,
    grid: Grid,
}

impl Puzzle {
    fn max_horz_clue_len(&self) -> usize {
        self.horz_clues
            .0
            .iter()
            .map(|clue| clue.0.len())
            .max()
            .unwrap()
    }
    fn max_vert_clue_len(&self) -> usize {
        self.vert_clues
            .0
            .iter()
            .map(|clue| clue.0.len())
            .max()
            .unwrap()
    }
    pub fn try_from_ast(ast: parser::Puzzle) -> Result<Puzzle, String> {
        let w = ast.vert_clues.0.len();
        let h = ast.horz_clues.0.len();
        if let Some(grid) = ast.grid {
            for (i, grid_line) in grid.0.iter().enumerate() {
                if w != grid_line.0.len() {
                    return Err(format!(
                        "number of vertical clues not same as number of grid columns in grid_line {} ({} vs {})", i + 1, w, grid_line.0.len()));
                }
            }
            if h != grid.0.len() {
                return Err(format!(
                    "number of horizontal clues not same as number of grid lines ({} vs {})",
                    h,
                    grid.0.len()
                ));
            }
            let mut filled = FixedBitSet::with_capacity(w * h);
            let mut crossed = FixedBitSet::with_capacity(w * h);
            let mut i = 0;
            for grid_line in grid.0 {
                for cell in grid_line.0 {
                    match cell {
                        Cell::Filled => {
                            filled.put(i);
                        }
                        Cell::Crossed => {
                            crossed.put(i);
                        }
                        Cell::Impossible => {
                            filled.put(i);
                            crossed.put(i);
                        }
                        _ => {}
                    }
                    i += 1;
                }
            }
            Ok(Puzzle {
                vert_clues: ast.vert_clues,
                horz_clues: ast.horz_clues,
                grid: Grid {
                    width: w,
                    height: h,
                    filled,
                    crossed,
                },
            })
        } else {
            let filled = FixedBitSet::with_capacity(w * h);
            let crossed = FixedBitSet::with_capacity(w * h);
            Ok(Puzzle {
                vert_clues: ast.vert_clues,
                horz_clues: ast.horz_clues,
                grid: Grid {
                    width: w,
                    height: h,
                    filled,
                    crossed,
                },
            })
        }
    }
    pub fn into_ast_without_grid(self) -> parser::Puzzle {
        parser::Puzzle {
            horz_clues: self.horz_clues,
            vert_clues: self.vert_clues,
            grid: None,
        }
    }
    pub fn into_ast(self) -> parser::Puzzle {
        let h = self.horz_clues.0.len();
        let w = self.vert_clues.0.len();
        let mut grid_lines = Vec::with_capacity(w);
        for y in 0..h {
            let cells = (0..w).map(|x| self.grid.get(x, y)).collect();
            grid_lines.push(GridLine(cells));
        }
        parser::Puzzle {
            horz_clues: self.horz_clues,
            vert_clues: self.vert_clues,
            grid: Some(parser::Grid(grid_lines)),
        }
    }
}

impl fmt::Display for Puzzle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let w = self.vert_clues.0.len();
        let max_vert_clue_len = self.max_vert_clue_len();
        let max_horz_clue_len = self.max_horz_clue_len();
        for i in 0..max_vert_clue_len {
            write!(f, "{: >width$}", "", width = 3 * max_horz_clue_len)?;
            for clue in &self.vert_clues.0 {
                if clue.0.len() > max_vert_clue_len - i - 1 {
                    write!(f, "{: >2}", clue.0[clue.0.len() - (max_vert_clue_len - i)])?;
                } else {
                    write!(f, "  ")?;
                }
            }
            write!(f, "\n")?;
        }
        for (y, clue) in self.horz_clues.0.iter().enumerate() {
            for i in 0..max_horz_clue_len {
                if clue.0.len() > max_horz_clue_len - i - 1 {
                    write!(f, " {: >2}", clue.0[clue.0.len() - (max_horz_clue_len - i)])?;
                } else {
                    write!(f, "   ")?;
                }
            }
            for x in 0..w {
                let ch = match self.grid.get(x, y) {
                    Cell::Crossed => '⨉',
                    Cell::Filled => '■',
                    Cell::Impossible => '!',
                    Cell::Undecided => '·',
                };
                write!(f, " {}", ch)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}
