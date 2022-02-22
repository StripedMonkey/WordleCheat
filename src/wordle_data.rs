use crate::{
    file_operations::{read_permutation_file, SingleGuess, WordPossibilities},
    information_theory,
    word_stats::{self, is_valid_guess, CharacterRENAMEME},
};
use indicatif::ParallelProgressIterator;
use ordered_float::NotNan;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rustc_hash::FxHashMap;
use std::sync::RwLock;

pub(crate) struct Game<'a> {
    wordle_data: WordleData<'a>,
    guess_stats: Vec<GuessData<'a>>,
}

impl<'dict> Game<'dict> {
    pub fn new(
        weight_map: Option<FxHashMap<&'dict str, f64>>,
        dictionary: Option<&'dict Vec<&str>>,
        permutation_file_name: &str,
        // frequency_file_name: &str,
    ) -> Game<'dict> {
        let weight_map = match weight_map {
            Some(wm) => wm,
            None => todo!(),
        };

        let dictionary = match dictionary {
            Some(dictionary) => dictionary,
            None => todo!(),
        };
        Game {
            wordle_data: WordleData::new(weight_map, dictionary, permutation_file_name),
            guess_stats: Vec::new(),
        }
    }

    pub fn possible_words(&self) -> Vec<&'dict str> {
        self.wordle_data.get_possible_words()
    }

    pub fn remaining_words(&self) -> usize {
        self.wordle_data.remaining_words()
    }

    pub fn add_information(&mut self, character_information: CharacterRENAMEME) {
        self.wordle_data.add_information(character_information)
    }

    pub fn eliminate_words(&mut self) -> usize {
        self.wordle_data.eliminate_words()
    }

    pub fn evaluate_information(&mut self, guess: &str) -> GuessData {
        let estimated_entropy = self.wordle_data.estimated_entropy(guess);

        let original_count: f64 = self.remaining_words() as f64;
        let new_count: f64 = original_count - self.eliminate_words() as f64;
        let actual_entropy = (original_count / new_count).log2();
        let guess_data = GuessData {
            estimated_entropy,
            actual_entropy,
            possible_answers: Some(self.possible_words()),
            guess: guess.to_string(),
        };
        self.guess_stats.push(guess_data.clone());
        guess_data
    }

    pub fn prioritize_position(&mut self) {
        self.wordle_data.prioritize_position()
    }
    pub fn prioritize_entropy(&mut self) {
        self.wordle_data.prioritize_entropy()
    }
}

struct WordleData<'dict> {
    dictionary: Vec<&'dict str>,
    weight_dictionary: FxHashMap<&'dict str, f64>,
    dictionary_weights: FxHashMap<&'dict str, f64>,
    permutation_map: FxHashMap<String, WordPossibilities>,
    weight_sum: f64,
    data_bits: Vec<CharacterRENAMEME>,
}

impl<'dict> WordleData<'dict> {
    pub fn new(
        weights: FxHashMap<&'dict str, f64>,
        dictionary: &'dict Vec<&str>,
        permutation_file_name: &str,
    ) -> WordleData<'dict> {
        let dictionary_weights: FxHashMap<&'dict str, f64> =
            generate_dict_weights_map(Some(&weights), dictionary);
        let mut permutation_map = read_permutation_file(permutation_file_name).possibilities;
        let sum = dictionary_weights.values().sum::<f64>();

        for word in dictionary {
            match permutation_map.get(*word) {
                Some(_) => (),
                None => {
                    println!("Warning! Permutation Map does not contain {word}! Calculating...");
                    permutation_map.insert(
                        { *word }.to_string(),
                        information_theory::generate_possibilities(*word),
                    );
                }
            }
        }
        Self {
            dictionary: dictionary.to_vec(), // CHECK: Is this right?
            weight_sum: sum,
            dictionary_weights,
            permutation_map,
            weight_dictionary: weights,
            data_bits: Vec::default(),
        }
    }

    pub fn get_possible_words(&self) -> Vec<&'dict str> {
        self.dictionary.clone()
    }

    pub fn remaining_words(&self) -> usize {
        self.dictionary.len()
    }

    pub fn add_information(&mut self, character_information: CharacterRENAMEME) {
        self.data_bits.push(character_information)
    }

    fn estimated_entropy(&self, guess: &str) -> f64 {
        let distribution = self.calculate_entropy_distribution(guess);
        distribution.iter().sum::<f64>() // TODO: This should be just the sum? See video
    }

    fn get_pattern_words(&self, validation: &SingleGuess) -> Vec<&str> {
        let lst = self
            .dictionary
            .par_iter()
            .filter_map(|s| {
                if is_valid_guess(&validation.guess, s) {
                    Some(*s)
                } else {
                    None
                }
            })
            .collect();
        lst
    }

    fn calculate_pattern_probability(&self, validation: &SingleGuess) -> f64 {
        let prob = self
            .get_pattern_words(validation)
            .iter()
            .map(|f| self.weight_dictionary.get(*f).unwrap_or(&0_f64))
            .sum::<f64>();
        // TODO: Come up with a slightly more reasonable way to do this.
        prob / self.weight_sum
    }

    fn calculate_entropy_distribution(&self, guess: &str) -> Vec<f64> {
        self.permutation_map
            .get(guess)
            .unwrap()
            // TODO: It's annoyingly difficult to do error handling here. How might this be done?
            // .unwrap_or_else(|| {
            // eprint!(
            // "Error! Word \"{guess}\" was not in the dictionary. Calculating on the fly."
            // );
            // self.permutation_map.write().unwrap().insert(
            // guess.to_string(),
            // information_theory::generate_possibilities(guess),
            // );
            // let x = self.permutation_map
            // .read()
            // .unwrap()
            // .get(guess)
            // .expect("Just added word to dictionary yet it's not there?");
            // x
            // })
            .guesses
            .iter()
            .map(|pattern| {
                let probability = self.calculate_pattern_probability(pattern);
                let entropy = information_theory::calculate_pattern_entropy(probability);
                let information = probability * entropy;
                if information.is_nan() {
                    return 0.0;
                }
                information
            })
            .collect()
    }

    pub fn eliminate_words(&mut self) -> usize {
        self.dictionary
            .drain_filter(|word| !word_stats::is_valid_guess(&self.data_bits, word))
            .count()
    }

    fn calculate_expected_entropy(&self, guess: &str) -> f64 {
        self.calculate_entropy_distribution(guess)
            .iter()
            .sum::<f64>()
    }

    // Sort `self.dictionary` based on the entropy that each word has
    pub fn prioritize_entropy(&mut self) {
        let entropy_map: FxHashMap<&str, f64> = self
            .dictionary
            .par_iter()        
            .progress_with(
                indicatif::ProgressBar::new(self.dictionary.len() as u64)
                    .with_message("Entropy Calcuations")
                    .with_style(
                        indicatif::ProgressStyle::default_bar()
                            .template(
                                "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg} ({eta})",
                            )
                            .progress_chars("##-"),
                    ),
            )
            .map(|word| (*word, self.calculate_expected_entropy(word)))
            .collect();
        self.dictionary.sort_by_key(|s| {
            NotNan::new(*entropy_map.get(s).unwrap_or_else(|| {
                eprintln!("Warning: {s} not in entropy map. Defaulting to 0 entropy");
                &0.0
            }))
            .unwrap()
        });
        self.dictionary.reverse();
    }

    pub fn prioritize_position(&mut self) {}
}

#[derive(Clone, Debug)]
pub(crate) struct GuessData<'a> {
    pub guess: String,
    pub estimated_entropy: f64,
    pub actual_entropy: f64,
    pub possible_answers: Option<Vec<&'a str>>,
}

impl GuessData<'_> {}

fn generate_dict_weights_map<'a>(
    frequency_map: Option<&FxHashMap<&'a str, f64>>,
    dictionary: &'a Vec<&str>,
) -> FxHashMap<&'a str, f64> {
    let mut dict_weight_map: FxHashMap<&str, f64> = FxHashMap::default();
    match frequency_map {
        Some(frequency_map) => {
            for word in dictionary {
                dict_weight_map.insert(word, *frequency_map.get(*word).unwrap_or(&0.0_f64));
            }
        }
        None => {
            for word in dictionary {
                dict_weight_map.insert(word, 1.0_f64);
            }
        }
    }
    dict_weight_map
}
