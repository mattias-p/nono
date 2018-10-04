use pest::iterators::Pair;
use std::borrow::Cow;
use std::fmt;

#[derive(Parser)]
#[grammar = "nono.pest"]
pub struct NonoParser;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Clue(pub Vec<usize>);

impl<'a> From<Pair<'a, Rule>> for Clue {
    fn from(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::clue);
        Clue(
            pair.into_inner()
                .map(|n| usize::from_str_radix(n.as_str(), 10).unwrap())
                .collect::<Vec<_>>(),
        )
    }
}

impl fmt::Display for Clue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some((first, rest)) = self.0.split_first() {
            write!(f, "{}", first)?;
            for number in rest {
                write!(f, ",{}", number)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClueList(pub Vec<Clue>);

impl<'a> From<Pair<'a, Rule>> for ClueList {
    fn from(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::clue_list);
        ClueList(pair.into_inner().map(Clue::from).collect())
    }
}

impl fmt::Display for ClueList {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (first, rest) = self.0.split_first().unwrap();
        write!(f, "{}", first)?;
        for number in rest {
            write!(f, ";{}", number)?;
        }
        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Cell {
    Filled,
    Crossed,
    Undecided,
    Impossible,
}

impl<'a> From<Pair<'a, Rule>> for Cell {
    fn from(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::cell);
        match pair.into_inner().next().unwrap().as_rule() {
            Rule::filled => Cell::Filled,
            Rule::crossed => Cell::Crossed,
            Rule::undecided => Cell::Undecided,
            Rule::impossible => Cell::Impossible,
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Cell::Filled => write!(f, "#"),
            Cell::Crossed => write!(f, "x"),
            Cell::Undecided => write!(f, "."),
            Cell::Impossible => write!(f, "!"),
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct GridLine(pub Vec<Cell>);

impl<'a> From<Pair<'a, Rule>> for GridLine {
    fn from(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::grid_line);
        GridLine(pair.into_inner().map(Cell::from).collect())
    }
}

impl fmt::Display for GridLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for cell in &self.0 {
            write!(f, "{}", cell)?;
        }
        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Grid(pub Vec<GridLine>);

impl<'a> From<Pair<'a, Rule>> for Grid {
    fn from(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::grid);
        Grid(pair.into_inner().map(GridLine::from).collect())
    }
}

impl fmt::Display for Grid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (first, rest) = self.0.split_first().unwrap();
        write!(f, "{}", first)?;
        for grid_line in rest {
            write!(f, ";{}", grid_line)?;
        }
        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Puzzle<'a> {
    pub vert_clues: Cow<'a, ClueList>,
    pub horz_clues: Cow<'a, ClueList>,
    pub grid: Option<Grid>,
}

impl<'a> From<Pair<'a, Rule>> for Puzzle<'a> {
    fn from(pair: Pair<Rule>) -> Self {
        assert_eq!(pair.as_rule(), Rule::puzzle);
        let mut pairs = pair.into_inner();
        let vert_clues = Cow::Owned(pairs.next().map(ClueList::from).unwrap());
        let horz_clues = Cow::Owned(pairs.next().map(ClueList::from).unwrap());
        let grid = pairs.next().map(Grid::from);
        Puzzle {
            vert_clues,
            horz_clues,
            grid,
        }
    }
}

impl<'a> fmt::Display for Puzzle<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(grid) = &self.grid {
            write!(f, "[{}|{}|{}]", self.vert_clues, self.horz_clues, grid)
        } else {
            write!(f, "[{}|{}]", self.vert_clues, self.horz_clues)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pest::Parser;

    fn test_roundtrip<T, F>(f: F, orig: T)
    where
        F: Fn(&str) -> Vec<T>,
        T: fmt::Debug + fmt::Display + Eq,
    {
        let items = f(&format!("{}", &orig));
        assert_eq!(items.as_slice(), [orig]);
    }

    #[test]
    fn clue() {
        fn deser(s: &str) -> Vec<Clue> {
            NonoParser::parse(Rule::clue, s)
                .unwrap_or_else(|e| panic!("{}", e))
                .map(Clue::from)
                .collect()
        }
        test_roundtrip(deser, Clue(vec![]));
        test_roundtrip(deser, Clue(vec![10]));
        test_roundtrip(deser, Clue(vec![1, 3, 5]));
    }

    #[test]
    fn clue_list() {
        fn deser(s: &str) -> Vec<ClueList> {
            NonoParser::parse(Rule::clue_list, s)
                .unwrap_or_else(|e| panic!("{}", e))
                .map(ClueList::from)
                .collect()
        }
        test_roundtrip(deser, ClueList(vec![Clue(vec![1])]));
        test_roundtrip(deser, ClueList(vec![Clue(vec![1]), Clue(vec![2])]));
        test_roundtrip(deser, ClueList(vec![Clue(vec![]), Clue(vec![2])]));
        test_roundtrip(deser, ClueList(vec![Clue(vec![1]), Clue(vec![])]));
        test_roundtrip(
            deser,
            ClueList(vec![Clue(vec![1]), Clue(vec![2]), Clue(vec![3])]),
        );
        test_roundtrip(
            deser,
            ClueList(vec![Clue(vec![1]), Clue(vec![]), Clue(vec![3])]),
        );
    }

    #[test]
    fn cell() {
        fn deser(s: &str) -> Vec<Cell> {
            NonoParser::parse(Rule::cell, s)
                .unwrap_or_else(|e| panic!("{}", e))
                .map(Cell::from)
                .collect()
        }
        test_roundtrip(deser, Cell::Filled);
        test_roundtrip(deser, Cell::Crossed);
        test_roundtrip(deser, Cell::Undecided);
        test_roundtrip(deser, Cell::Impossible);
    }

    #[test]
    fn grid_line() {
        fn deser(s: &str) -> Vec<GridLine> {
            NonoParser::parse(Rule::grid_line, s)
                .unwrap_or_else(|e| panic!("{}", e))
                .map(GridLine::from)
                .collect()
        }
        test_roundtrip(deser, GridLine(vec![Cell::Undecided]));
        test_roundtrip(deser, GridLine(vec![Cell::Filled, Cell::Crossed]));
    }

    #[test]
    fn grid() {
        fn deser(s: &str) -> Vec<Grid> {
            NonoParser::parse(Rule::grid, s)
                .unwrap_or_else(|e| panic!("{}", e))
                .map(Grid::from)
                .collect()
        }
        test_roundtrip(deser, Grid(vec![GridLine(vec![Cell::Undecided])]));
        test_roundtrip(
            deser,
            Grid(vec![
                GridLine(vec![Cell::Undecided, Cell::Filled]),
                GridLine(vec![Cell::Crossed, Cell::Impossible]),
            ]),
        );
    }

    #[test]
    fn puzzle() {
        fn deser(s: &str) -> Vec<Puzzle> {
            NonoParser::parse(Rule::puzzle, s)
                .unwrap_or_else(|e| panic!("{}", e))
                .map(Puzzle::from)
                .collect()
        }
        test_roundtrip(
            deser,
            Puzzle {
                vert_clues: Cow::Owned(ClueList(vec![Clue(vec![]), Clue(vec![1])])),
                horz_clues: Cow::Owned(ClueList(vec![Clue(vec![1]), Clue(vec![])])),
                grid: None,
            },
        );
        test_roundtrip(
            deser,
            Puzzle {
                vert_clues: Cow::Owned(ClueList(vec![Clue(vec![]), Clue(vec![1])])),
                horz_clues: Cow::Owned(ClueList(vec![Clue(vec![1]), Clue(vec![])])),
                grid: Some(Grid(vec![
                    GridLine(vec![Cell::Undecided, Cell::Filled]),
                    GridLine(vec![Cell::Crossed, Cell::Impossible]),
                ])),
            },
        );
    }
}
