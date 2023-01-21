use std::{path::PathBuf, str::FromStr};

#[derive(Debug, Clone, Copy)]
pub enum Day {
    Day01,
    Day02,
    Day03,
    Day04,
    Day05,
    Day06,
    Day07,
    Day08,
    Day09,
    Day10,
    Day11,
    Day12,
    Day13,
    Day14,
    Day15,
    Day16,
    Day17,
    Day18,
    Day19,
    Day20,
    Day21,
    Day22,
    Day23,
    Day24,
    Day25,
}

impl Day {
    fn to_numeric(self) -> u8 {
        match self {
            Day::Day01 => 1,
            Day::Day02 => 2,
            Day::Day03 => 3,
            Day::Day04 => 4,
            Day::Day05 => 5,
            Day::Day06 => 6,
            Day::Day07 => 7,
            Day::Day08 => 8,
            Day::Day09 => 9,
            Day::Day10 => 10,
            Day::Day11 => 11,
            Day::Day12 => 12,
            Day::Day13 => 13,
            Day::Day14 => 14,
            Day::Day15 => 15,
            Day::Day16 => 16,
            Day::Day17 => 17,
            Day::Day18 => 18,
            Day::Day19 => 19,
            Day::Day20 => 20,
            Day::Day21 => 21,
            Day::Day22 => 22,
            Day::Day23 => 23,
            Day::Day24 => 24,
            Day::Day25 => 25,
        }
    }

    pub fn to_web_input_path(self, base_path: &str) -> String {
        format!("{}{}/input", base_path, self.to_numeric())
    }

    pub fn to_filename(self) -> PathBuf {
        PathBuf::from_str(&format!("{}.in", self.to_numeric())).expect("Could not generate filename from path")
    }
}
