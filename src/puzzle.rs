use fixedbitset::FixedBitSet;
use std::borrow::Borrow;
use std::borrow::Cow;
use std::fmt;
use std::ops::Range;
use std::str::FromStr;

use parser;
use parser::Cell;
use parser::ClueList;
use parser::GridLine;

pub trait LineHint: fmt::Debug {
    fn check(&self, line: &Line) -> bool;
    fn apply(&self, line: &mut LineMut);
}

#[derive(Clone, Copy, Debug)]
pub enum Axis {
    Horz,
    Vert,
}

static ORIENTATIONS: [Axis; 2] = [Axis::Horz, Axis::Vert];

impl Axis {
    pub fn get(index: usize) -> Option<Self> {
        ORIENTATIONS.get(index).cloned()
    }
}

#[derive(Debug)]
pub struct Hint<H: LineHint> {
    axis: Axis,
    line: usize,
    line_hint: Box<H>,
}

impl<H: LineHint> Hint<H> {
    pub fn apply<'a>(&self, puzzle: &mut Puzzle<'a>) {
        match self.axis {
            Axis::Vert => {
                self.line_hint.apply(&mut VertLineMut {
                    grid: &mut puzzle.grid,
                    x: self.line,
                });
            }
            Axis::Horz => {
                self.line_hint.apply(&mut HorzLineMut {
                    grid: &mut puzzle.grid,
                    y: self.line,
                });
            }
        }
    }
}

pub trait LinePass: fmt::Debug {
    type Hint: LineHint;
    fn run(&self, clue: &[usize], line: &Line) -> Vec<Box<Self::Hint>>;
}

pub trait LinePassExt<H: LineHint> {
    fn run_vert(&self, puzzle: &Puzzle) -> Vec<Hint<H>>;
    fn run_horz(&self, puzzle: &Puzzle) -> Vec<Hint<H>>;
    fn run_puzzle(&self, axis: &Axis, puzzle: &Puzzle) -> Vec<Hint<H>> {
        match axis {
            Axis::Vert => self.run_vert(puzzle),
            Axis::Horz => self.run_horz(puzzle),
        }
    }
    fn apply(&self, axis: &Axis, puzzle: &mut Puzzle) -> Vec<Hint<H>> {
        let hints = self.run_puzzle(axis, puzzle);
        for hint in &hints {
            hint.apply(puzzle);
        }
        // println!( "\nAfter {:?} line:\n{}", axis, Theme::Unicode.view(puzzle));
        hints
    }
}

impl<H: LineHint, T: LinePass<Hint = H>> LinePassExt<H> for T {
    fn run_vert(&self, puzzle: &Puzzle) -> Vec<Hint<H>> {
        let mut hints = vec![];
        for (x, clue) in puzzle.vert_clues.0.iter().enumerate() {
            let mut line = VertLine {
                grid: &puzzle.grid,
                x,
            };
            for line_hint in self.run(clue.0.as_slice(), &line) {
                let hint = Hint {
                    axis: Axis::Vert,
                    line: x,
                    line_hint,
                };
                hints.push(hint);
            }
        }
        hints
    }

    fn run_horz(&self, puzzle: &Puzzle) -> Vec<Hint<H>> {
        let mut hints = vec![];
        for (y, clue) in puzzle.horz_clues.0.iter().enumerate() {
            let mut line = HorzLine {
                grid: &puzzle.grid,
                y,
            };
            for line_hint in self.run(clue.0.as_slice(), &line) {
                let hint = Hint {
                    axis: Axis::Horz,
                    line: y,
                    line_hint,
                };
                hints.push(hint);
            }
        }
        hints
    }
}

pub trait Line {
    fn len(&self) -> usize;
    fn get(&self, i: usize) -> Cell;
    fn is_crossed(&self, i: usize) -> bool;
    fn is_filled(&self, i: usize) -> bool;

    fn range_contains_filled(&self, r: Range<usize>) -> bool {
        for i in r {
            if self.is_filled(i) {
                return true;
            }
        }
        false
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

    fn bump_start(&self, start: usize, number: usize) -> usize {
        //println!("BUMP START {} {}", start, number);
        //if start > 0 { println!("  check filled {}", start - 1); }
        let mut start = if start > 0 && self.is_filled(start - 1) {
            //println!("  pushed");
            start + 1
        } else {
            start
        };
        let mut focus = start;

        while focus < start + number {
            //println!("  check crossed {}", focus);
            if focus < self.len() && self.is_crossed(focus) {
                //println!("  pushed");
                start = focus + 1;
            }
            focus += 1;
        }
        //println!("  check filled {}", focus);
        while focus < self.len() && self.is_filled(focus) {
            focus += 1;
            //println!("  pulled");
            //println!("  check filled {}", focus);
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

        assert!(focus + number < self.len() as isize);
        focus + number + 1
    }
}

pub trait LineExt: Line {
    fn view(&self) -> LineView;
}

impl<T: Line> LineExt for T {
    fn view(&self) -> LineView {
        LineView(self)
    }
}

pub struct LineView<'a>(&'a Line);

impl<'a> fmt::Display for LineView<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.0.len() {
            write!(f, "{}", self.0.get(i))?;
        }
        Ok(())
    }
}

pub trait LineMut: Line {
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
}

pub struct HorzLine<'a> {
    grid: &'a Grid,
    y: usize,
}

impl<'a> Line for HorzLine<'a> {
    fn get(&self, x: usize) -> Cell {
        self.grid.get(x, self.y)
    }
    fn is_crossed(&self, x: usize) -> bool {
        self.grid.is_crossed(x, self.y)
    }
    fn is_filled(&self, x: usize) -> bool {
        self.grid.is_filled(x, self.y)
    }
    fn len(&self) -> usize {
        self.grid.width
    }
}

pub struct HorzLineMut<'a> {
    grid: &'a mut Grid,
    y: usize,
}

impl<'a> Line for HorzLineMut<'a> {
    fn get(&self, x: usize) -> Cell {
        self.grid.get(x, self.y)
    }
    fn is_crossed(&self, x: usize) -> bool {
        self.grid.is_crossed(x, self.y)
    }
    fn is_filled(&self, x: usize) -> bool {
        self.grid.is_filled(x, self.y)
    }
    fn len(&self) -> usize {
        self.grid.width
    }
}

impl<'a> LineMut for HorzLineMut<'a> {
    fn cross(&mut self, x: usize) {
        self.grid.cross(x, self.y);
    }
    fn fill(&mut self, x: usize) {
        self.grid.fill(x, self.y);
    }
}

pub struct VertLine<'a> {
    grid: &'a Grid,
    x: usize,
}

impl<'a> Line for VertLine<'a> {
    fn get(&self, y: usize) -> Cell {
        self.grid.get(self.x, y)
    }
    fn is_crossed(&self, y: usize) -> bool {
        self.grid.is_crossed(self.x, y)
    }
    fn is_filled(&self, y: usize) -> bool {
        self.grid.is_filled(self.x, y)
    }
    fn len(&self) -> usize {
        self.grid.height
    }
}

pub struct VertLineMut<'a> {
    grid: &'a mut Grid,
    x: usize,
}

impl<'a> Line for VertLineMut<'a> {
    fn get(&self, y: usize) -> Cell {
        self.grid.get(self.x, y)
    }
    fn is_crossed(&self, y: usize) -> bool {
        self.grid.is_crossed(self.x, y)
    }
    fn is_filled(&self, y: usize) -> bool {
        self.grid.is_filled(self.x, y)
    }
    fn len(&self) -> usize {
        self.grid.height
    }
}

impl<'a> LineMut for VertLineMut<'a> {
    fn cross(&mut self, y: usize) {
        self.grid.cross(self.x, y);
    }
    fn fill(&mut self, y: usize) {
        self.grid.fill(self.x, y);
    }
}

pub struct Grid {
    width: usize,
    height: usize,
    filled: FixedBitSet,
    crossed: FixedBitSet,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        Grid {
            width,
            height,
            filled: FixedBitSet::with_capacity(width * height),
            crossed: FixedBitSet::with_capacity(width * height),
        }
    }
    pub fn horz_mut(&mut self, y: usize) -> HorzLineMut {
        HorzLineMut { grid: self, y }
    }
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

pub struct Puzzle<'a> {
    vert_clues: Cow<'a, ClueList>,
    horz_clues: Cow<'a, ClueList>,
    grid: Grid,
}

impl<'a> Puzzle<'a> {
    pub fn is_complete(&self) -> bool {
        for i in 0..self.grid.filled.len() {
            if !self.grid.filled.contains(i) && !self.grid.crossed.contains(i) {
                return false;
            }
        }
        true
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
    pub fn try_from_ast(ast: parser::Puzzle<'a>) -> Result<Puzzle<'a>, String> {
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

    #[allow(dead_code)]
    pub fn into_ast_without_grid(self) -> parser::Puzzle<'a> {
        parser::Puzzle {
            horz_clues: self.horz_clues,
            vert_clues: self.vert_clues,
            grid: None,
        }
    }

    pub fn as_ast(&self) -> parser::Puzzle {
        let h = self.horz_clues.0.len();
        let w = self.vert_clues.0.len();
        let mut grid_lines = Vec::with_capacity(w);
        for y in 0..h {
            let cells = (0..w).map(|x| self.grid.get(x, y)).collect();
            grid_lines.push(GridLine(cells));
        }
        parser::Puzzle {
            horz_clues: Cow::Borrowed(self.horz_clues.borrow()),
            vert_clues: Cow::Borrowed(self.vert_clues.borrow()),
            grid: Some(parser::Grid(grid_lines)),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Theme {
    Ascii,
    Unicode,
    Brief,
}

impl Theme {
    pub fn crossed(&self) -> char {
        match self {
            Theme::Ascii => '.',
            Theme::Unicode => '⨉',
            Theme::Brief => 'E',
        }
    }

    pub fn filled(&self) -> char {
        match self {
            Theme::Ascii => '#',
            Theme::Unicode => '■',
            Theme::Brief => 'E',
        }
    }

    pub fn impossible(&self) -> char {
        match self {
            Theme::Ascii => '!',
            Theme::Unicode => '!',
            Theme::Brief => 'E',
        }
    }

    pub fn undecided(&self) -> char {
        match self {
            Theme::Ascii => ' ',
            Theme::Unicode => '·',
            Theme::Brief => 'E',
        }
    }

    pub fn view<'a>(&'a self, puzzle: &'a Puzzle) -> View<'a> {
        View {
            puzzle,
            theme: self,
        }
    }
}

impl FromStr for Theme {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ascii" => Ok(Theme::Ascii),
            "unicode" => Ok(Theme::Unicode),
            "brief" => Ok(Theme::Brief),
            _ => Err("unrecognized theme"),
        }
    }
}

pub struct View<'a> {
    puzzle: &'a Puzzle<'a>,
    theme: &'a Theme,
}

impl<'a> fmt::Display for View<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if *self.theme == Theme::Brief {
            return self.puzzle.as_ast().fmt(f);
        }

        let w = self.puzzle.vert_clues.0.len();
        let max_vert_clue_len = self.puzzle.max_vert_clue_len();
        let max_horz_clue_len = self.puzzle.max_horz_clue_len();
        for i in 0..max_vert_clue_len {
            write!(f, "{: >width$}", "", width = 3 * max_horz_clue_len)?;
            for clue in &self.puzzle.vert_clues.0 {
                if clue.0.len() > max_vert_clue_len - i - 1 {
                    write!(f, "{: >2}", clue.0[clue.0.len() - (max_vert_clue_len - i)])?;
                } else {
                    write!(f, "  ")?;
                }
            }
            writeln!(f)?;
        }
        for (y, clue) in self.puzzle.horz_clues.0.iter().enumerate() {
            for i in 0..max_horz_clue_len {
                if clue.0.len() > max_horz_clue_len - i - 1 {
                    write!(f, " {: >2}", clue.0[clue.0.len() - (max_horz_clue_len - i)])?;
                } else {
                    write!(f, "   ")?;
                }
            }
            for x in 0..w {
                let ch = match self.puzzle.grid.get(x, y) {
                    Cell::Crossed => self.theme.crossed(),
                    Cell::Filled => self.theme.filled(),
                    Cell::Impossible => self.theme.impossible(),
                    Cell::Undecided => self.theme.undecided(),
                };
                write!(f, " {}", ch)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

pub fn line_grid(s: &str) -> Grid {
    use parser::NonoParser;
    use parser::Rule;
    use pest::Parser;
    let s = format!("[{}||{}]", ";".repeat(s.len() - 1), s);
    let ast = NonoParser::parse(Rule::puzzle, &s)
        .unwrap_or_else(|e| panic!("{}", e))
        .next()
        .map(parser::Puzzle::from)
        .unwrap();
    Puzzle::try_from_ast(ast).unwrap().grid
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bump_start_empty() {
        let mut grid = Grid::new(10, 1);
        let line = grid.horz_mut(0);
        assert_eq!(line.bump_start(0, 3), 0);
    }

    #[test]
    fn bump_start_one_filled() {
        let mut grid = Grid::new(10, 1);
        let mut line = grid.horz_mut(0);
        line.fill(4);
        assert_eq!(line.bump_start(0, 3), 0);
        assert_eq!(line.bump_start(1, 3), 2);
        assert_eq!(line.bump_start(2, 3), 2);
        assert_eq!(line.bump_start(3, 3), 3);
        assert_eq!(line.bump_start(4, 3), 4);
        assert_eq!(line.bump_start(5, 3), 6);
        assert_eq!(line.bump_start(6, 3), 6);
        assert_eq!(line.bump_start(7, 3), 7);
    }

    #[test]
    fn bump_start_one_crossed() {
        let mut grid = Grid::new(10, 1);
        let mut line = grid.horz_mut(0);
        line.cross(4);
        assert_eq!(line.bump_start(0, 3), 0);
        assert_eq!(line.bump_start(1, 3), 1);
        assert_eq!(line.bump_start(2, 3), 5);
        assert_eq!(line.bump_start(3, 3), 5);
        assert_eq!(line.bump_start(4, 3), 5);
        assert_eq!(line.bump_start(5, 3), 5);
        assert_eq!(line.bump_start(6, 3), 6);
        assert_eq!(line.bump_start(7, 3), 7);
    }

    #[test]
    fn bump_start_two_filled() {
        let mut grid = Grid::new(4, 1);
        let mut line = grid.horz_mut(0);
        line.fill(0);
        line.fill(2);
        assert_eq!(line.bump_start(0, 1), 0);
        assert_eq!(line.bump_start(1, 1), 2);
        assert_eq!(line.bump_start(2, 1), 2);
        assert_eq!(line.bump_start(3, 1), 4);
    }

    #[test]
    fn bump_start_number_two() {
        use puzzle::Grid;

        let mut grid = Grid::new(6, 1);
        let mut line = grid.horz_mut(0);
        line.fill(0);
        line.cross(2);
        line.fill(4);
        line.cross(5);
        assert_eq!(line.bump_start(2, 2), 3);
        assert_eq!(line.bump_start(3, 2), 3);
        assert_eq!(line.bump_start(4, 2), 6);
    }
}
