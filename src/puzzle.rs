use fixedbitset::FixedBitSet;
use parser::Cell;
use parser::ClueList;
use parser::GridLine;
use std::fmt;
use std::ops::Range;

use parser;

pub trait LinePass {
    fn run(&self, clue: &[usize], line: &mut Line);
}

pub trait LinePassExt {
    fn apply_horz(&self, puzzle: &mut Puzzle);
    fn apply_vert(&self, puzzle: &mut Puzzle);
}

impl<T: LinePass> LinePassExt for T {
    fn apply_horz(&self, puzzle: &mut Puzzle) {
        for (y, clue) in puzzle.horz_clues.0.iter().enumerate() {
            self.run(
                clue.0.as_slice(),
                &mut HorzLine {
                    grid: &mut puzzle.grid,
                    y,
                },
            );
        }
    }
    fn apply_vert(&self, puzzle: &mut Puzzle) {
        for (x, clue) in puzzle.vert_clues.0.iter().enumerate() {
            self.run(
                clue.0.as_slice(),
                &mut VertLine {
                    grid: &mut puzzle.grid,
                    x,
                },
            );
        }
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
        self.grid.fill(x, self.y);
    }
    fn fill(&mut self, x: usize) {
        self.grid.cross(x, self.y);
    }
    fn cross_range(&mut self, xs: Range<usize>) {
        self.grid.cross_horz(xs, self.y);
    }
    fn fill_range(&mut self, xs: Range<usize>) {
        self.grid.fill_horz(xs, self.y);
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
        self.grid.fill(self.x, y);
    }
    fn fill(&mut self, y: usize) {
        self.grid.cross(self.x, y);
    }
    fn cross_range(&mut self, ys: Range<usize>) {
        self.grid.cross_vert(self.x, ys);
    }
    fn fill_range(&mut self, ys: Range<usize>) {
        self.grid.fill_vert(self.x, ys);
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
    fn fill_horz(&mut self, xs: Range<usize>, y: usize) {
        for x in xs {
            self.fill(x, y);
        }
    }
    fn fill_vert(&mut self, x: usize, ys: Range<usize>) {
        for y in ys {
            self.fill(x, y);
        }
    }
    fn fill(&mut self, x: usize, y: usize) {
        let i = self.index(x, y);
        if !self.filled.contains(i) {
            //println!("fill {} {} {}", x, y, transposed);
        }
        self.filled.put(i);
    }
    fn cross(&mut self, x: usize, y: usize) {
        let i = self.index(x, y);
        if !self.crossed.contains(i) {
            //println!("cross {} {} {}", x, y, transposed);
        }
        self.crossed.put(i);
    }
    fn cross_horz(&mut self, xs: Range<usize>, y: usize) {
        for x in xs {
            self.cross(x, y);
        }
    }
    fn cross_vert(&mut self, x: usize, ys: Range<usize>) {
        for y in ys {
            self.cross(x, y);
        }
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
                write!(f, " {}", self.grid.get(x, y))?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}