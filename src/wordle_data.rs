use rustc_hash::FxHashMap;

use crate::{
    information_theory::calculate_entropy_distribution,
    word_stats::{self, CharacterRENAMEME},
};

pub(crate) struct Game<'a> {
    wordle_data: WordleData<'a>,
    guess_stats: Vec<GuessData<'a>>,
}

impl<'dict> Game<'dict> {
    pub fn new(
        weight_map: Option<FxHashMap<&'dict str, f64>>,
        dictionary: Option<&'dict Vec<&str>>,
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
            wordle_data: WordleData::new(weight_map, dictionary),
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
        let actual_entropy = (original_count/new_count).log2();
        let guess_data = GuessData {
            estimated_entropy,
            actual_entropy,
            possible_answers: Some(self.possible_words()),
            guess: guess.to_string(),
        };
        self.guess_stats.push(guess_data.clone());
        guess_data
    }
}

struct WordleData<'dict> {
    dictionary: Vec<&'dict str>,
    weight_dictionary: FxHashMap<&'dict str, f64>,
    dictionary_weights: FxHashMap<&'dict str, f64>,
    weight_sum: f64,
    data_bits: Vec<CharacterRENAMEME>,
}

impl<'dict> WordleData<'dict> {
    pub fn new(
        weights: FxHashMap<&'dict str, f64>,
        dictionary: &'dict Vec<&str>,
    ) -> WordleData<'dict> {
        let dictionary_weights: FxHashMap<&'dict str, f64> =
            generate_dict_weights_map(Some(&weights), dictionary);
        let sum = dictionary_weights.values().sum::<f64>();
        Self {
            dictionary: dictionary.to_vec(), // CHECK: Is this right?
            weight_sum: sum,
            dictionary_weights,
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
        let distribution =
            calculate_entropy_distribution(guess, &self.dictionary, &self.dictionary_weights);
        distribution.iter().sum::<f64>() // TODO: This should be just the sum? See video
    }

    pub fn eliminate_words(&mut self) -> usize {
        self.dictionary
            .drain_filter(|word| !word_stats::is_valid_guess(&self.data_bits, word))
            .count()
    }
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
