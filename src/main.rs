extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate fixedbitset;

mod parser;

use fixedbitset::FixedBitSet;
use parser::Cell;
use parser::ClueList;
use parser::Grid;
use parser::GridLine;
use parser::NonoParser;
use parser::Rule;
use pest::Parser;
use std::fmt;
use std::io;
use std::io::BufRead;

struct Puzzle {
    vert_clues: ClueList,
    horz_clues: ClueList,
    filled: FixedBitSet,
    crossed: FixedBitSet,
}

impl Puzzle {
    fn try_from_ast(ast: parser::Puzzle) -> Result<Puzzle, String> {
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
                filled,
                crossed,
            })
        } else {
            let filled = FixedBitSet::with_capacity(w * h);
            let crossed = FixedBitSet::with_capacity(w * h);
            Ok(Puzzle {
                vert_clues: ast.vert_clues,
                horz_clues: ast.horz_clues,
                filled,
                crossed,
            })
        }
    }
    fn into_ast_without_grid(self) -> parser::Puzzle {
        parser::Puzzle {
            horz_clues: self.horz_clues,
            vert_clues: self.vert_clues,
            grid: None,
        }
    }
    fn into_ast(self) -> parser::Puzzle {
        let h = self.horz_clues.0.len();
        let w = self.vert_clues.0.len();
        let mut grid_lines = Vec::with_capacity(w);
        for y in 0..h {
            let cells = (0..w).map(|x| self.get_xy(x, y)).collect();
            grid_lines.push(GridLine(cells));
        }
        parser::Puzzle {
            horz_clues: self.horz_clues,
            vert_clues: self.vert_clues,
            grid: Some(Grid(grid_lines)),
        }
    }
    fn get(&self, i: usize) -> Cell {
        match (self.filled.contains(i), self.crossed.contains(i)) {
            (false, false) => Cell::Undecided,
            (false, true) => Cell::Crossed,
            (true, false) => Cell::Filled,
            (true, true) => Cell::Impossible,
        }
    }
    fn index(&self, x: usize, y: usize) -> usize {
        y * self.horz_clues.0.len() + x
    }
    fn get_xy(&self, x: usize, y: usize) -> Cell {
        self.get(self.index(x, y))
    }
    fn width(&self) -> usize {
        self.vert_clues.0.len()
    }
    fn height(&self) -> usize {
        self.horz_clues.0.len()
    }
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
                    write!(f, "{: >2}", clue.0[max_vert_clue_len - i - 1])?;
                } else {
                    write!(f, "  ")?;
                }
            }
            write!(f, "\n")?;
        }
        for (y, clue) in self.horz_clues.0.iter().enumerate() {
            for i in 0..max_horz_clue_len {
                if clue.0.len() > max_horz_clue_len - i - 1 {
                    write!(f, " {: >2}", clue.0[max_horz_clue_len - i - 1])?;
                } else {
                    write!(f, "   ")?;
                }
            }
            for x in 0..w {
                write!(f, " {}", self.get_xy(x, y))?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

trait Pass {
    fn run(puzzle: &mut Puzzle);
}

struct BasicFreedom;

impl Pass for BasicFreedom {
    fn run(puzzle: &mut Puzzle) {
        let horz_clues = &puzzle.horz_clues.0;
        for (y, clue) in horz_clues.iter().enumerate() {
            let sum: usize = clue.0.iter().sum();
            let freedom: usize = puzzle.width() - (sum + clue.0.len() - 1);
            let mut x0 = 0;
            for number in clue.0.iter() {
                if *number > freedom {
                    for x1 in x0 + freedom..x0 + number {
                        let i = puzzle.index(x1, y);
                        puzzle.filled.put(i);
                    }
                }
                x0 += number + 1;
            }
        }
        for (x, clue) in puzzle.vert_clues.0.iter().enumerate() {
            let sum: usize = clue.0.iter().sum();
            let freedom: usize = puzzle.height() - (sum + clue.0.len() - 1);
            let mut y0 = 0;
            for number in clue.0.iter() {
                if *number > freedom {
                    for y1 in y0 + freedom..y0 + number {
                        let i = puzzle.index(x, y1);
                        puzzle.filled.put(i);
                    }
                }
                y0 += number + 1;
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
        match Puzzle::try_from_ast(ast) {
            Ok(mut puzzle) => {
                BasicFreedom::run(&mut puzzle);
                println!("{}", puzzle);
            }
            Err(e) => panic!("{}", e),
        }
    }
}
