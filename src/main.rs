extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate fixedbitset;

mod parser;

use fixedbitset::FixedBitSet;
use parser::Cell;
use parser::Clue;
use parser::ClueList;
use parser::GridLine;
use parser::NonoParser;
use parser::Rule;
use pest::Parser;
use std::fmt;
use std::io;
use std::io::BufRead;
use std::iter;
use std::ops::Range;

struct Clues {
    vert: ClueList,
    horz: ClueList,
}

impl Clues {
    fn horz(&self, transposed: bool) -> &[Clue] {
        if transposed {
            self.vert.0.as_slice()
        } else {
            self.horz.0.as_slice()
        }
    }
    fn vert(&self, transposed: bool) -> &[Clue] {
        self.horz(!transposed)
    }
    fn width(&self, transposed: bool) -> usize {
        if transposed {
            self.horz.0.len()
        } else {
            self.vert.0.len()
        }
    }
    fn height(&self, transposed: bool) -> usize {
        self.width(!transposed)
    }
    fn max_horz_len(&self, transposed: bool) -> usize {
        let clue_list = if transposed { &self.vert } else { &self.horz };
        clue_list.0.iter().map(|clue| clue.0.len()).max().unwrap()
    }
    fn max_vert_len(&self, transposed: bool) -> usize {
        self.max_horz_len(!transposed)
    }
}

struct Grid {
    width: usize,
    height: usize,
    filled: FixedBitSet,
    crossed: FixedBitSet,
}

impl Grid {
    fn index(&self, x: usize, y: usize, transposed: bool) -> usize {
        if transposed {
            assert!(x < self.height);
            assert!(y < self.width);
            x * self.height + y
        } else {
            assert!(x < self.width);
            assert!(y < self.height);
            y * self.width + x
        }
    }
    fn get(&self, x: usize, y: usize, transposed: bool) -> Cell {
        let i = self.index(x, y, transposed);
        match (self.filled.contains(i), self.crossed.contains(i)) {
            (false, false) => Cell::Undecided,
            (false, true) => Cell::Crossed,
            (true, false) => Cell::Filled,
            (true, true) => Cell::Impossible,
        }
    }
    fn fill_horz(&mut self, xs: Range<usize>, y: usize, transposed: bool) {
        for x in xs {
            self.fill(x, y, transposed);
        }
    }
    fn fill(&mut self, x: usize, y: usize, transposed: bool) {
        let i = self.index(x, y, transposed);
        if !self.filled.contains(i) {
            //println!("fill {} {} {}", x, y, transposed);
        }
        self.filled.put(i);
    }
    fn cross(&mut self, x: usize, y: usize, transposed: bool) {
        let i = self.index(x, y, transposed);
        if !self.crossed.contains(i) {
            //println!("cross {} {} {}", x, y, transposed);
        }
        self.crossed.put(i);
    }
    fn cross_horz(&mut self, xs: Range<usize>, y: usize, transposed: bool) {
        for x in xs {
            self.cross(x, y, transposed);
        }
    }
    fn is_crossed(&self, x: usize, y: usize, transposed: bool) -> bool {
        let i = self.index(x, y, transposed);
        self.crossed.contains(i)
    }
    fn is_filled(&self, x: usize, y: usize, transposed: bool) -> bool {
        let i = self.index(x, y, transposed);
        self.filled.contains(i)
    }
}

struct Puzzle {
    clues: Clues,
    grid: Grid,
}

impl Puzzle {
    fn apply(&mut self, pass: &impl Pass) {
        Pass::run(pass, self, true);
        Pass::run(pass, self, false);
    }
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
                clues: Clues {
                    vert: ast.vert_clues,
                    horz: ast.horz_clues,
                },
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
                clues: Clues {
                    vert: ast.vert_clues,
                    horz: ast.horz_clues,
                },
                grid: Grid {
                    width: w,
                    height: h,
                    filled,
                    crossed,
                },
            })
        }
    }
    fn into_ast_without_grid(self) -> parser::Puzzle {
        parser::Puzzle {
            horz_clues: self.clues.horz,
            vert_clues: self.clues.vert,
            grid: None,
        }
    }
    fn into_ast(self) -> parser::Puzzle {
        let h = self.clues.horz.0.len();
        let w = self.clues.vert.0.len();
        let mut grid_lines = Vec::with_capacity(w);
        for y in 0..h {
            let cells = (0..w).map(|x| self.grid.get(x, y, false)).collect();
            grid_lines.push(GridLine(cells));
        }
        parser::Puzzle {
            horz_clues: self.clues.horz,
            vert_clues: self.clues.vert,
            grid: Some(parser::Grid(grid_lines)),
        }
    }
}

impl fmt::Display for Puzzle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let w = self.clues.vert.0.len();
        let max_vert_clue_len = self.clues.max_vert_len(false);
        let max_horz_clue_len = self.clues.max_horz_len(false);
        for i in 0..max_vert_clue_len {
            write!(f, "{: >width$}", "", width = 3 * max_horz_clue_len)?;
            for clue in &self.clues.vert.0 {
                if clue.0.len() > max_vert_clue_len - i - 1 {
                    write!(f, "{: >2}", clue.0[clue.0.len() - (max_vert_clue_len - i)])?;
                } else {
                    write!(f, "  ")?;
                }
            }
            write!(f, "\n")?;
        }
        for (y, clue) in self.clues.horz.0.iter().enumerate() {
            for i in 0..max_horz_clue_len {
                if clue.0.len() > max_horz_clue_len - i - 1 {
                    write!(f, " {: >2}", clue.0[clue.0.len() - (max_horz_clue_len - i)])?;
                } else {
                    write!(f, "   ")?;
                }
            }
            for x in 0..w {
                write!(f, " {}", self.grid.get(x, y, false))?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

trait Pass {
    fn run(&self, puzzle: &mut Puzzle, transposed: bool);
}

struct BasicFreedom;

impl Pass for BasicFreedom {
    fn run(&self, puzzle: &mut Puzzle, transposed: bool) {
        for (y, clue) in puzzle.clues.horz.0.iter().enumerate() {
            let sum: usize = clue.0.iter().sum();
            let freedom: usize = puzzle.clues.width(transposed) - (sum + clue.0.len() - 1);
            let mut x0 = 0;
            for number in clue.0.iter() {
                if *number > freedom {
                    puzzle
                        .grid
                        .fill_horz(x0 + freedom..x0 + number, y, transposed);
                }
                x0 += number + 1;
            }
        }
    }
}

struct Freedom2;

impl Pass for Freedom2 {
    fn run(&self, puzzle: &mut Puzzle, transposed: bool) {
        let w = puzzle.clues.width(transposed);
        for (y, clue) in puzzle.clues.horz(transposed).iter().enumerate() {
            //println!("{}", &puzzle);
            //println!("\nclue1  {}", &clue);
            let mut min_starts = Vec::with_capacity(clue.0.len());
            let mut x0 = 0;
            for number in clue.0.iter() {
                let mut x = x0;
                while x < w - number && x < x0 + number {
                    if puzzle.grid.is_crossed(x, y, transposed) {
                        //println!("start pushed by cross at {}", x);
                        // pushing cross
                        x0 = x + 1;
                    }
                    x += 1;
                }
                if x + 1 < w && puzzle.grid.is_filled(x + 1, y, transposed) {
                    // pulling fill
                    while x + 1 < w && puzzle.grid.is_filled(x, y, transposed) {
                        //println!("start pulled by fill at {}", x);
                        x += 1;
                    }
                    // TODO check for impossibility
                    x0 = x.max(*number) - number;
                }
                assert!(x0 <= w - number);
                //println!("min start {}", x0);
                min_starts.push(x0);
                x0 += 1 + number;
            }
            let mut max_ends = Vec::with_capacity(clue.0.len());
            let mut x1 = w + 1;
            for number in clue.0.iter().rev() {
                //println!("rev number {}", number);
                x1 -= 1;
                let mut x = x1 - 1;
                while x > 0 && x > x1 - number {
                    if puzzle.grid.is_crossed(x, y, transposed) {
                        //println!("end pushed by cross at {}", x);
                        // pushing cross
                        x1 = x;
                    }
                    x -= 1;
                }

                if x > 0 && puzzle.grid.is_filled(x - 1, y, transposed) {
                    // pulling fill
                    while x > 0 && puzzle.grid.is_filled(x - 1, y, transposed) {
                        //println!("end pulled by fill at {}", x - 1);
                        x -= 1;
                    }
                    // TODO check for impossibility
                    x1 = x + number;
                }
                //println!("max end {}", x1);
                assert!(x1 <= w);
                assert!(x1 >= *number);
                max_ends.push(x1);
                x1 -= number;
            }
            //println!("\nclue2  {}", &clue);
            for ((((number, min_start), max_end), prev_max_end), next_min_start) in clue
                .0
                .iter()
                .zip(min_starts.iter())
                .zip(max_ends.iter().rev())
                .zip(iter::once(&0).chain(max_ends.iter().rev()))
                .zip(min_starts.iter().skip(1).chain(iter::once(&w)))
            {
                assert!(min_start + number <= *max_end);
                //println!(
                //"{}  {}/{}/{}  {}",
                //prev_max_end, min_start, number, max_end, next_min_start,
                //);
                if *max_end == *min_start + number {
                    if *min_start > 0 {
                        puzzle.grid.cross(min_start - 1, y, transposed);
                    }
                    puzzle.grid.fill_horz(*min_start..*max_end, y, transposed);
                    if *max_end < w {
                        puzzle.grid.cross(*max_end, y, transposed);
                    }
                    continue;
                }
                if *min_start + 2 * number > *max_end {
                    //println!("middle {}..{}", max_end - number, min_start + number);
                    puzzle
                        .grid
                        .fill_horz(max_end - number..min_start + number, y, transposed);

                    /*
                    if let Some(x0) = (*prev_max_end.max(min_start)..max_end - number)
                        .find(|x| puzzle.grid.is_filled(*x, y, transposed))
                    {
                        println!("found {}", x0);
                        puzzle.grid.fill_horz(x0..max_end - number, y, transposed);
                        puzzle.grid.cross_horz(x0 + number..*max_end, y, transposed);
                    }
                    if let Some(x1) = (min_start + number..*next_min_start.min(max_end))
                        .rev()
                        .find(|x| puzzle.grid.is_filled(*x, y, transposed))
                    {
                        println!("found {}", x1);
                        puzzle.grid.fill_horz(min_start + number..x1, y, transposed);
                        puzzle
                            .grid
                            .cross_horz(*min_start..x1 - number, y, transposed);
                    }
                    */
                }
                /*
                if let Some(x0) = (*prev_max_end.max(min_start)..*max_end.min(next_min_start))
                    .find(|x| puzzle.grid.is_filled(*x, y, transposed))
                {
                    if x0 > *number {
                        puzzle.grid.cross_horz(
                            *prev_max_end.max(min_start)..x0 - number,
                            y,
                            transposed,
                        );
                    }
                    if let Some(x1) = (x0..*max_end.min(next_min_start))
                        .rev()
                        .find(|x| puzzle.grid.is_filled(*x, y, transposed))
                    {
                        puzzle.grid.cross_horz(
                            *max_end.min(next_min_start)..x1.max(*number) - number,
                            y,
                            transposed,
                        );
                    }
                }
                */
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
                puzzle.apply(&Freedom2);
                println!("{}", puzzle);
                //puzzle.apply(&Freedom2);
                //println!("{}", puzzle);
                //puzzle.apply(&Freedom2);
                //println!("{}", puzzle);
                //puzzle.apply(&Freedom2);
                //println!("{}", puzzle);
            }
            Err(e) => panic!("{}", e),
        }
    }
}
