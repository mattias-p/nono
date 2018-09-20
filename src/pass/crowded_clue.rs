use puzzle::Line;
use puzzle::LineHint;
use puzzle::LinePass;

#[derive(Debug)]
pub struct CrowdedClue {
    kernel_start: usize,
    kernel_end: usize,
}

impl LineHint for CrowdedClue {
    fn check(&self, line: &Line) -> bool {
        line.range_contains_unfilled(self.kernel_start..self.kernel_end)
    }
    fn apply(&self, line: &mut Line) {
        line.fill_range(self.kernel_start..self.kernel_end);
    }
}

pub struct CrowdedCluePass;

impl LinePass for CrowdedCluePass {
    type Hint = CrowdedClue;
    fn run(&self, clue: &[usize], line: &Line) -> Vec<Box<Self::Hint>> {
        let mut hints: Vec<Box<Self::Hint>> = vec![];
        let sum: usize = clue.iter().sum();
        let freedom: usize = line.len() - (sum + clue.len() - 1);
        let mut x0 = 0;
        for number in clue.iter() {
            if *number > freedom {
                let hint = Box::new(CrowdedClue {
                    kernel_start: x0 + freedom,
                    kernel_end: x0 + number,
                });
                if hint.check(line) {
                    hints.push(hint);
                }
            }
            x0 += number + 1;
        }
        hints
    }
}
