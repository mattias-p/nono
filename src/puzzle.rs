use fixedbitset::FixedBitSet;
use parser::Cell;
use parser::ClueList;
use parser::GridLine;
use std::fmt;
use std::ops::Range;

use parser;

#[derive(Debug)]
pub enum LineHint {
    CrowdedClue {
        kernel_start: usize,
        kernel_end: usize,
    },
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

impl LineHint {
    pub fn check(&self, line: &Line) -> bool {
        match self {
            &LineHint::CrowdedClue {
                kernel_start,
                kernel_end,
            } => line.range_contains_unfilled(kernel_start..kernel_end),
            &LineHint::Unreachable {
                reachable_start,
                reachable_end,
            } => {
                let len = line.len();
                line.range_contains_uncrossed(0..reachable_start)
                    || line.range_contains_uncrossed(reachable_end..len)
            }
            &LineHint::Kernel {
                kernel_start,
                kernel_end,
            } => line.range_contains_unfilled(kernel_start..kernel_end),
            &LineHint::Termination {
                range_start,
                range_end,
            } => {
                (range_start > 0 && !line.is_crossed(range_start - 1))
                    || (range_end < line.len() && !line.is_crossed(range_end))
            }
            &LineHint::TurfNearSingleton {
                found_start,
                kernel_start,
                reachable_end,
                turf_end,
            } => {
                line.range_contains_unfilled(found_start..kernel_start)
                    || line.range_contains_uncrossed(reachable_end..turf_end)
            }
            &LineHint::TurfFarSingleton {
                turf_start,
                reachable_start,
                kernel_end,
                found_end,
            } => {
                line.range_contains_uncrossed(turf_start..reachable_start)
                    || line.range_contains_unfilled(kernel_end..found_end)
            }
            &LineHint::TurfPair {
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
            &LineHint::TurfSingleton {
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
    pub fn apply(&self, line: &mut Line) {
        match self {
            &LineHint::CrowdedClue {
                kernel_start,
                kernel_end,
            } => {
                line.fill_range(kernel_start..kernel_end);
            }
            &LineHint::Unreachable {
                reachable_start,
                reachable_end,
            } => {
                let len = line.len();
                line.cross_range(0..reachable_start);
                line.cross_range(reachable_end..len);
            }
            &LineHint::Kernel {
                kernel_start,
                kernel_end,
            } => {
                line.fill_range(kernel_start..kernel_end);
            }
            &LineHint::Termination {
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
            &LineHint::TurfNearSingleton {
                found_start,
                kernel_start,
                reachable_end,
                turf_end,
            } => {
                line.fill_range(found_start..kernel_start);
                line.cross_range(reachable_end..turf_end);
            }
            &LineHint::TurfFarSingleton {
                turf_start,
                reachable_start,
                kernel_end,
                found_end,
            } => {
                line.cross_range(turf_start..reachable_start);
                line.fill_range(kernel_end..found_end);
            }
            &LineHint::TurfPair {
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
            &LineHint::TurfSingleton {
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

#[derive(Debug)]
enum Orientation {
    Horz,
    Vert,
}

#[derive(Debug)]
pub struct Hint {
    orientation: Orientation,
    line: usize,
    line_hint: LineHint,
}

pub trait LinePass {
    fn run(&self, clue: &[usize], line: &Line) -> Vec<LineHint>;
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
                is_dirty: false,
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
                is_dirty: false,
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
    fn is_crossed(&self, i: usize) -> bool;
    fn is_filled(&self, i: usize) -> bool;
    fn cross(&mut self, i: usize);
    fn fill(&mut self, i: usize);
    fn cross_range(&mut self, r: Range<usize>);
    fn fill_range(&mut self, r: Range<usize>);
    fn len(&self) -> usize;
    fn check_dirty(&mut self) -> bool;
    fn range_contains_unfilled(&self, r: Range<usize>) -> bool;
    fn range_contains_uncrossed(&self, r: Range<usize>) -> bool;
}

struct HorzLine<'a> {
    grid: &'a mut Grid,
    y: usize,
    is_dirty: bool,
}

impl<'a> Line for HorzLine<'a> {
    fn is_crossed(&self, x: usize) -> bool {
        self.grid.is_crossed(x, self.y)
    }
    fn is_filled(&self, x: usize) -> bool {
        self.grid.is_filled(x, self.y)
    }
    fn cross(&mut self, x: usize) {
        if self.grid.cross(x, self.y) {
            self.is_dirty = true;
        }
    }
    fn fill(&mut self, x: usize) {
        if self.grid.fill(x, self.y) {
            self.is_dirty = true;
        }
    }
    fn cross_range(&mut self, xs: Range<usize>) {
        if self.grid.cross_horz(xs, self.y) {
            self.is_dirty = true;
        }
    }
    fn fill_range(&mut self, xs: Range<usize>) {
        if self.grid.fill_horz(xs, self.y) {
            self.is_dirty = true;
        }
    }
    fn len(&self) -> usize {
        self.grid.width
    }
    fn check_dirty(&mut self) -> bool {
        if self.is_dirty {
            self.is_dirty = false;
            true
        } else {
            false
        }
    }
    fn range_contains_unfilled(&self, xs: Range<usize>) -> bool {
        for x in xs {
            if !self.grid.is_filled(x, self.y) {
                return true;
            }
        }
        false
    }
    fn range_contains_uncrossed(&self, xs: Range<usize>) -> bool {
        for x in xs {
            if !self.grid.is_crossed(x, self.y) {
                return true;
            }
        }
        false
    }
}

struct VertLine<'a> {
    grid: &'a mut Grid,
    x: usize,
    is_dirty: bool,
}

impl<'a> Line for VertLine<'a> {
    fn is_crossed(&self, y: usize) -> bool {
        self.grid.is_crossed(self.x, y)
    }
    fn is_filled(&self, y: usize) -> bool {
        self.grid.is_filled(self.x, y)
    }
    fn cross(&mut self, y: usize) {
        if self.grid.cross(self.x, y) {
            self.is_dirty = true;
        }
    }
    fn fill(&mut self, y: usize) {
        if self.grid.fill(self.x, y) {
            self.is_dirty = true;
        }
    }
    fn cross_range(&mut self, ys: Range<usize>) {
        if self.grid.cross_vert(self.x, ys) {
            self.is_dirty = true;
        }
    }
    fn fill_range(&mut self, ys: Range<usize>) {
        if self.grid.fill_vert(self.x, ys) {
            self.is_dirty = true;
        }
    }
    fn len(&self) -> usize {
        self.grid.height
    }
    fn check_dirty(&mut self) -> bool {
        if self.is_dirty {
            self.is_dirty = false;
            true
        } else {
            false
        }
    }
    fn range_contains_unfilled(&self, ys: Range<usize>) -> bool {
        for y in ys {
            if !self.grid.is_filled(self.x, y) {
                return true;
            }
        }
        false
    }
    fn range_contains_uncrossed(&self, ys: Range<usize>) -> bool {
        for y in ys {
            if !self.grid.is_crossed(self.x, y) {
                return true;
            }
        }
        false
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
    fn fill_horz(&mut self, xs: Range<usize>, y: usize) -> bool {
        let mut is_dirty = false;
        for x in xs {
            if self.fill(x, y) {
                is_dirty = true;
            }
        }
        is_dirty
    }
    fn fill_vert(&mut self, x: usize, ys: Range<usize>) -> bool {
        let mut is_dirty = false;
        for y in ys {
            if self.fill(x, y) {
                is_dirty = true;
            }
        }
        is_dirty
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
    fn cross_horz(&mut self, xs: Range<usize>, y: usize) -> bool {
        let mut is_dirty = false;
        for x in xs {
            if self.cross(x, y) {
                is_dirty = true;
            }
        }
        is_dirty
    }
    fn cross_vert(&mut self, x: usize, ys: Range<usize>) -> bool {
        let mut is_dirty = false;
        for y in ys {
            if self.cross(x, y) {
                is_dirty = true;
            }
        }
        is_dirty
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
