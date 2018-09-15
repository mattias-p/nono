extern crate pest;
#[macro_use]
extern crate pest_derive;
extern crate fixedbitset;

mod parser;

use fixedbitset::FixedBitSet;
use parser::Cell;
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

struct Puzzle {
    clues: Clues,
    grid: Grid,
}

impl Puzzle {
    fn max_horz_clue_len(&self) -> usize {
        self.clues
            .horz
            .0
            .iter()
            .map(|clue| clue.0.len())
            .max()
            .unwrap()
    }
    fn max_vert_clue_len(&self) -> usize {
        self.clues
            .vert
            .0
            .iter()
            .map(|clue| clue.0.len())
            .max()
            .unwrap()
    }
    fn line_pass(&mut self, pass: &impl LinePass) {
        for (y, clue) in self.clues.horz.0.iter().enumerate() {
            pass.run(
                clue.0.as_slice(),
                &mut HorzLine {
                    grid: &mut self.grid,
                    y,
                },
            );
        }
        for (x, clue) in self.clues.vert.0.iter().enumerate() {
            pass.run(
                clue.0.as_slice(),
                &mut VertLine {
                    grid: &mut self.grid,
                    x,
                },
            );
        }
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
            let cells = (0..w).map(|x| self.grid.get(x, y)).collect();
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
        let max_vert_clue_len = self.max_vert_clue_len();
        let max_horz_clue_len = self.max_horz_clue_len();
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
                write!(f, " {}", self.grid.get(x, y))?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

trait LinePass {
    fn run(&self, clue: &[usize], line: &mut Line);
}

struct BasicFreedom;

impl LinePass for BasicFreedom {
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

trait Line {
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

struct Freedom2;

impl LinePass for Freedom2 {
    fn run(&self, clue: &[usize], line: &mut Line) {
        let w = line.len();
        //println!("{}", &puzzle);
        //println!("CLUE  {}", line.clue);

        let mut range_starts = Vec::with_capacity(clue.len());
        let mut x0 = 0;
        for number in clue.iter() {
            let mut x = x0;
            while x < w && x < x0 + number {
                if line.is_crossed(x) {
                    // pushing cross
                    x0 = x + 1;
                }
                x += 1;
            }
            if x < w && line.is_filled(x) {
                // pulling fill
                while x < w && line.is_filled(x) {
                    x += 1;
                }
                // TODO check for impossibility
                x0 = x - number;
            }
            assert!(x0 <= w - number);
            range_starts.push(x0);
            x0 += number;
        }

        let mut range_ends = Vec::with_capacity(clue.len());
        let mut x1 = w + 1;
        for number in clue.iter().rev() {
            x1 -= 1;
            let mut x = x1 - 1;
            while x > 0 && x > x1 - number {
                if line.is_crossed(x) {
                    // pushing cross
                    x1 = x;
                }
                x -= 1;
            }

            if x > 0 && line.is_filled(x - 1) {
                // pulling fill
                while x > 0 && line.is_filled(x - 1) {
                    x -= 1;
                }
                // TODO check for impossibility
                x1 = x + number;
            }
            assert!(x1 <= w);
            assert!(x1 >= *number);
            range_ends.push(x1);
            x1 -= number;
        }

        line.cross_range(0..range_starts[0]);
        line.cross_range(range_ends[0]..w);

        for ((((number, range_start), range_end), prev_range_end), next_min_start) in clue
            .iter()
            .zip(range_starts.iter())
            .zip(range_ends.iter().rev())
            .zip(iter::once(&0).chain(range_ends.iter().rev()))
            .zip(range_starts.iter().skip(1).chain(iter::once(&w)))
        {
            assert!(range_start + number <= *range_end);

            let turf_start = *prev_range_end.max(range_start);
            let turf_end = *range_end.min(next_min_start);

            //println!("number {}", number);
            //println!("range  {}..{}", range_start, range_end);
            //println!("turf   {}..{}", turf_start, turf_end);

            if *range_end == *range_start + number {
                if *range_start > 0 {
                    line.cross(range_start - 1);
                }
                line.fill_range(*range_start..*range_end);
                if *range_end < w {
                    line.cross(*range_end);
                }
                continue;
            }

            if *range_start + 2 * number > *range_end {
                let kernel_start = range_end - number;
                let kernel_end = range_start + number;

                //println!("kernel {}..{}", kernel_start, kernel_end);

                line.fill_range(kernel_start..kernel_end);

                if let Some(x0) = (turf_start..kernel_start).find(|x| line.is_filled(*x)) {
                    line.fill_range(x0..kernel_start);
                    line.cross_range(x0 + number..turf_end);
                }

                if let Some(x1) = (kernel_end..turf_end).rev().find(|x| line.is_filled(*x)) {
                    line.fill_range(kernel_end..x1);
                    line.cross_range(turf_start..x1 - number);
                }
            } else {
                //println!("{} {} - - {} {}", range_start, turf_start, turf_end, range_end);
                if let Some(x0) = (turf_start..turf_end).find(|x| line.is_filled(*x)) {
                    //println!("x0 {}", x0);
                    line.cross_range(x0 + number..turf_end);
                    line.cross_range(turf_start..(x0 + 1).max(*number) - number);

                    if let Some(x1) = (x0..turf_end).rev().find(|x| line.is_filled(*x)) {
                        line.fill_range(x0 + 1..x1);
                        line.cross_range(turf_start..x1.max(*number) - number);
                    } else {
                        line.cross_range(turf_start..x0.max(*number) - number);
                    }
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
        match Puzzle::try_from_ast(ast) {
            Ok(mut puzzle) => {
                for _x in 0..15 {
                    puzzle.line_pass(&Freedom2);
                    println!("{}", puzzle);
                }
                println!("{}", puzzle.into_ast());
            }
            Err(e) => panic!("{}", e),
        }
    }
}
