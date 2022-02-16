use std::fs::read_to_string;

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::word_stats::CharacterRENAMEME;

#[derive(Serialize, Deserialize, Debug)]
struct PrecomputedDictionary {
    possibilities: Vec<WordPossibilities>,
}

#[derive(Serialize, Deserialize, Debug)]
struct WordPossibilities {
    guesses: Vec<SingleGuess>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SingleGuess {
    guess: Vec<CharacterRENAMEME>,
}

// Quick and dirty reading of frequency tables
pub(crate) fn read_frequencytable_file(filename: &str) -> FxHashMap<String, f64> {
    read_to_string(filename)
        .unwrap()
        .lines()
        .map(|line| {
            let mut split = line.split_ascii_whitespace();
            (
                split.next().unwrap().to_string(),
                split.next().unwrap().parse::<f64>().unwrap(),
            )
        })
        .collect()
}
