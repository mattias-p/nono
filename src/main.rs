use std::num::ParseIntError;
use std::str::FromStr;

struct Clue {
    inner: Vec<usize>,
}

impl FromStr for Clue {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = s
            .split(' ')
            .map(|n| usize::from_str_radix(n, 10))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Clue { inner })
    }
}

fn main() {
    println!("Hello, world!");
}
