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
    fn apply(&self, line: &mut Line);
}

#[derive(Debug)]
pub enum Orientation {
    Horz,
    Vert,
}

static ORIENTATIONS: [Orientation; 2] = [Orientation::Horz, Orientation::Vert];

impl Orientation {
    pub fn all() -> &'static [Orientation; 2] {
        &ORIENTATIONS
    }
}

#[derive(Debug)]
pub struct Hint<H: LineHint> {
    orientation: Orientation,
    line: usize,
    line_hint: Box<H>,
}

impl<H: LineHint> Hint<H> {
    fn apply<'a>(&self, puzzle: &mut Puzzle<'a>) {
        match self.orientation {
            Orientation::Vert => {
                self.line_hint.apply(&mut VertLine {
                    grid: &mut puzzle.grid,
                    x: self.line,
                });
            }
            Orientation::Horz => {
                self.line_hint.apply(&mut HorzLine {
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
    fn apply_horz(&self, puzzle: &mut Puzzle) -> Vec<Hint<H>>;
    fn apply_vert(&self, puzzle: &mut Puzzle) -> Vec<Hint<H>>;
    fn apply(&self, orientation: &Orientation, puzzle: &mut Puzzle) -> Vec<Hint<H>> {
        match orientation {
            Orientation::Horz => self.apply_horz(puzzle),
            Orientation::Vert => self.apply_vert(puzzle),
        }
    }
}

impl<H: LineHint, T: LinePass<Hint = H>> LinePassExt<H> for T {
    fn apply_horz(&self, puzzle: &mut Puzzle) -> Vec<Hint<H>> {
        let mut hints = vec![];
        for (y, clue) in puzzle.horz_clues.0.iter().enumerate() {
            let mut line = HorzLine {
                grid: &mut puzzle.grid,
                y,
            };
            for line_hint in self.run(clue.0.as_slice(), &line) {
                let hint = Hint {
                    orientation: Orientation::Horz,
                    line: y,
                    line_hint,
                };
                hints.push(hint);
            }
        }
        for hint in &hints {
            hint.apply(puzzle);
        }
        //println!("\nAfter horz line:\n{}", puzzle);
        hints
    }
    fn apply_vert(&self, puzzle: &mut Puzzle) -> Vec<Hint<H>> {
        let mut hints = vec![];
        for (x, clue) in puzzle.vert_clues.0.iter().enumerate() {
            let mut line = VertLine {
                grid: &mut puzzle.grid,
                x,
            };
            for line_hint in self.run(clue.0.as_slice(), &line) {
                let hint = Hint {
                    orientation: Orientation::Vert,
                    line: x,
                    line_hint,
                };
                hints.push(hint);
            }
        }
        for hint in &hints {
            hint.apply(puzzle);
        }
        //println!("\nAfter vert line:\n{}", puzzle);
        hints
    }
}

struct ReverseLine(Box<Line>);

impl Line for ReverseLine {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn get(&self, i: usize) -> Cell {
        self.0.get(i)
    }
    fn is_crossed(&self, i: usize) -> bool {
        let i = self.0.len() - 1 - i;
        self.0.is_crossed(i)
    }
    fn is_filled(&self, i: usize) -> bool {
        let i = self.0.len() - 1 - i;
        self.0.is_filled(i)
    }
    fn cross(&mut self, i: usize) {
        let i = self.0.len() - 1 - i;
        self.0.cross(i)
    }
    fn fill(&mut self, i: usize) {
        let i = self.0.len() - 1 - i;
        self.0.fill(i)
    }
}

pub trait Line {
    fn len(&self) -> usize;
    fn get(&self, i: usize) -> Cell;
    fn is_crossed(&self, i: usize) -> bool;
    fn is_filled(&self, i: usize) -> bool;
    fn cross(&mut self, i: usize);
    fn fill(&mut self, i: usize);

    fn rev(self: Box<Self>) -> ReverseLine
    where
        Self: 'static + Sized,
    {
        ReverseLine(self)
    }

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

        assert!(focus + number + 1 <= self.len() as isize);
        focus + number + 1
    }
}

pub struct HorzLine<'a> {
    grid: &'a mut Grid,
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

pub struct VertLine<'a> {
    grid: &'a mut Grid,
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
    pub fn horz_mut(&mut self, y: usize) -> HorzLine {
        HorzLine { grid: self, y }
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

    pub fn into_ast(&self) -> parser::Puzzle {
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
            return self.puzzle.into_ast().fmt(f);
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
            write!(f, "\n")?;
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
            write!(f, "\n")?;
        }
        Ok(())
    }
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
