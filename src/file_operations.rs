use std::{
    fs::{read_to_string, File},
    io::{BufReader, Write},
};

use indicatif::ParallelProgressIterator;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::{information_theory, word_stats::CharacterRENAMEME};

// Framework for (de)serializing word combinations. Warning: this will be pretty large
#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct PrecomputedDictionary {
    pub possibilities: FxHashMap<String, WordPossibilities>,
}

impl FromIterator<(String, WordPossibilities)> for PrecomputedDictionary {
    fn from_iter<T: IntoIterator<Item = (String, WordPossibilities)>>(iter: T) -> Self {
        let mut dict: FxHashMap<String, WordPossibilities> = FxHashMap::default();
        dict.extend(iter);
        Self {
            possibilities: dict,
        }
    }
}

impl From<Vec<SingleGuess>> for WordPossibilities {
    fn from(guesses: Vec<SingleGuess>) -> Self {
        Self { guesses }
    }
}
impl From<Vec<CharacterRENAMEME>> for SingleGuess {
    fn from(guess: Vec<CharacterRENAMEME>) -> Self {
        Self { guess }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct WordPossibilities {
    pub guesses: Vec<SingleGuess>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct SingleGuess {
    pub guess: Vec<CharacterRENAMEME>,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct WeightData {
    pub data: FxHashMap<String, f64>,
}

// Quick and dirty reading of frequency tables
fn read_frequencytable_file(filename: &str) -> FxHashMap<String, f64> {
    match read_to_string(filename) {
        Ok(s) => s
            .lines()
            .map(|line| {
                let mut split = line.split_ascii_whitespace();
                (
                    split.next().unwrap().to_string(),
                    split.next().unwrap().parse::<f64>().unwrap(),
                )
            })
            .collect(),
        Err(e) => panic!("{}", e),
    }
}

// Generate the possible combinations of correct/incorrect
pub(crate) fn generate_permutation_file(frequency_map_file: &str, permutation_map_file: &str) {
    let frequency_map: FxHashMap<String, f64> =
        bincode::deserialize_from(File::open(frequency_map_file).unwrap()).unwrap();
    let mut permutation_map = File::create(permutation_map_file).unwrap();
    let perm_map: FxHashMap<String, WordPossibilities> = frequency_map
        .par_iter()
        .progress_with(
            indicatif::ProgressBar::new(frequency_map.len() as u64)
                .with_message("Total Progress")
                .with_style(
                    indicatif::ProgressStyle::default_bar()
                        .template(
                            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg} ({eta})",
                        )
                        .progress_chars("##-"),
                ),
        )
        .filter(|(s, _)| s.len() < 6)
        .map(|(s, _)| (s.clone(), information_theory::generate_possibilities(&s)))
        .collect();
    let dict = PrecomputedDictionary {
        possibilities: perm_map,
    };
    permutation_map
        .write(&bincode::serialize(&dict).expect("Failed to serialize dictionary!"))
        .expect("Failed to Write to dictionary!");
}

pub(crate) fn read_frequency_file(filename: &str) -> FxHashMap<String, f64> {
    let file = File::open(filename).unwrap();
    println!("Reading from file {filename}...");
    let map: FxHashMap<String, f64> = bincode::deserialize_from(file).unwrap();
    println!("Finished!");
    map
}

pub(crate) fn read_permutation_file(filename: &str) -> PrecomputedDictionary {
    let reader = BufReader::with_capacity(64 * 1024, File::open(filename).unwrap());
    println!("Reading from file {filename}...");
    let permutation_map: PrecomputedDictionary = bincode::deserialize_from(reader).unwrap();
    println!("Finished!");
    permutation_map
}

#[cfg(test)]
mod test {

    use super::read_frequencytable_file;
    use std::{fs::File, io::Write};

    #[test]
    // Generate a frequency for later use
    fn frequency_map() {
        let map = read_frequencytable_file("DictionaryWordWeight.txt");
        let mut file = File::create("FrequencyMap.bin").unwrap();
        file.write_all(bincode::serialize(&map).unwrap().as_slice());
    }
}
