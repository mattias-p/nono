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
        let mut i = 0;
        let h = self.horz_clues.0.len();
        let w = self.vert_clues.0.len();
        let mut grid_lines = Vec::with_capacity(w);
        for _y in 0..h {
            let mut cells = Vec::with_capacity(w);
            for _x in 0..w {
                let cell = match (self.filled.contains(i), self.crossed.contains(i)) {
                    (false, false) => Cell::Undecided,
                    (false, true) => Cell::Crossed,
                    (true, false) => Cell::Filled,
                    (true, true) => Cell::Impossible,
                };
                cells.push(cell);
                i += 1;
            }
            grid_lines.push(GridLine(cells));
        }
        parser::Puzzle {
            horz_clues: self.horz_clues,
            vert_clues: self.vert_clues,
            grid: Some(Grid(grid_lines)),
        }
    }
}

fn main() {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let pair = NonoParser::parse(Rule::puzzle, &line)
            .unwrap_or_else(|e| panic!("{}", e))
            .next()
            .unwrap();
        println!("{}", parser::Puzzle::from(pair));
    }
}
