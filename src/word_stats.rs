use rustc_hash::FxHashMap;

use indicatif::ParallelProgressIterator;
use ordered_float::NotNan;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::{Deserialize, Serialize};

use crate::{information_theory::calculate_expected_entropy, file_operations::read_frequency_file};

pub(crate) const WORDLESIZE: usize = 5;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub(crate) enum CharacterCorrectness {
    NotInWord(char),
    IncorrectPosition(char),
    CorrectPosition(char),
}

// TODO: Figure out what to call this. It's the CharacterCorrectness with positional information
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub(crate) struct CharacterRENAMEME {
    pub pos: usize,
    pub character: CharacterCorrectness,
}

pub(crate) struct CharFrequencies {
    pub frequency: usize,
    pub character: char,
}

pub(crate) struct CharPosition {
    pub position: usize,
    pub character: char,
}

#[derive(Debug)]
pub(crate) struct GuessResult<'dict, 'answer> {
    pub answer: &'answer str,
    pub path: Vec<&'dict str>,
}

pub(crate) type WordGuess = Vec<CharacterRENAMEME>;
pub(crate) type IncorrectPosition = CharPosition;
pub(crate) type CorrectPosition = CharPosition;

pub(crate) fn evaulate_uniqueness(unique_characters: &Vec<CharFrequencies>, word: &str) -> usize {
    let mut used = Vec::new();
    let mut value: usize = 0;
    for character in word.chars() {
        if !used.contains(&character) {
            used.push(character);
            let idx = unique_characters
                .binary_search_by_key(&character, |f| f.character)
                .expect("Couldn't find character in list!");
            value += unique_characters[idx].frequency;
        }
    }
    value
}

/// Generate a list of the frequency of each Character in a dictionary
pub(crate) fn character_frequencies(dictionary: &Vec<&str>) -> Vec<CharFrequencies> {
    let mut unique_characters_list: Vec<CharFrequencies> = Vec::new();
    for word in &*dictionary {
        // Count the number of unique characters in each word
        let mut unique_characters: Vec<char> = Vec::new();
        for character in word.chars() {
            if !unique_characters.contains(&character) {
                unique_characters.push(character);
            }
        }

        // Count the frequency of unique characters
        for character in unique_characters {
            match unique_characters_list.binary_search_by_key(&character, |f| f.character) {
                Ok(i) => {
                    unique_characters_list[i] = CharFrequencies {
                        // Should actually just increment this, but meh
                        character,
                        frequency: unique_characters_list[i].frequency + 1,
                    }
                }
                Err(i) => unique_characters_list.insert(
                    i,
                    CharFrequencies {
                        character,
                        frequency: 1,
                    },
                ),
            }
        }
    }
    unique_characters_list
}

/// Sort a dictionary by most frequent to least frequent characters within the dictionary
pub(crate) fn sort_dictionary_frequency(dictionary: &mut Vec<&str>) {
    // Sort by the frequency of unique characters in each word
    let unique_characters = character_frequencies(dictionary);
    dictionary.sort_by_cached_key(|word| evaulate_uniqueness(&unique_characters, word));
    dictionary.reverse();
}

/// Sort a dictionary by the most common locations for characters to least common
pub(crate) fn sort_dictionary_location(dictionary: &mut Vec<&str>) {
    let unique_positions = count_unique_positions(dictionary);
    dictionary.sort_by_key(|a| count_location(&unique_positions, a));
    // dictionary.reverse();
}

/// Sort a dictionary based on the entropy of a dictionary with progress bars
pub(crate) fn sort_dictionary_entropy_progress(
    // TODO: This should be moved into the wordle data because it's not
    // possible to calculate this without a lot of external data.
    guess_dictionary: &mut Vec<&str>,
    filtered_dictionary: &Vec<&str>,
) {
    let map: FxHashMap<&str, f64> = guess_dictionary
        .par_iter()
        .progress_with(
            indicatif::ProgressBar::new(filtered_dictionary.len() as u64)
                .with_message("Total Progress")
                .with_style(
                    indicatif::ProgressStyle::default_bar()
                        .template(
                            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg} ({eta})",
                        )
                        .progress_chars("##-"),
                ),
        )
        .map(|s| {
            (
                *s,
                calculate_expected_entropy(
                    s,
                    filtered_dictionary,
                    &generate_dict_weights_map(
                        Some(&read_frequency_file("count_1w.txt")),
                        guess_dictionary,
                    ),
                ),
            )
        })
        .collect();
    guess_dictionary.sort_by_cached_key(|s| NotNan::new(*map.get(s).unwrap()).unwrap());
    guess_dictionary.reverse();
}

pub(crate) fn count_location(positions: &FxHashMap<char, Vec<usize>>, word: &str) -> usize {
    let mut value: usize = 0;
    let mut searched_characters = Vec::new();
    for (i, character) in word.chars().enumerate() {
        if searched_characters.contains(&character) {
            searched_characters.push(character);
            value += positions.get(&character).expect("Couldn't find character!")[i];
        }
    }
    value
}

pub(crate) fn count_unique_positions(dictionary: &Vec<&str>) -> FxHashMap<char, Vec<usize>> {
    let mut unique_positions: FxHashMap<char, Vec<usize>> = FxHashMap::default();
    for word in &*dictionary {
        for (i, character) in word.chars().enumerate() {
            match unique_positions.get_mut(&character) {
                Some(arr) => arr[i] += 1,
                None => {
                    let mut positions = vec![0; WORDLESIZE];
                    positions[i] += 1;
                    unique_positions.insert(character, positions);
                }
            }
        }
    }
    unique_positions
}

/// Verifies that a particular word is a valid guess given a set of information
pub(crate) fn is_valid_guess(validation: &Vec<CharacterRENAMEME>, word: &str) -> bool {
    for correctness in validation {
        match correctness.character {
            CharacterCorrectness::NotInWord(c) => {
                if word.contains(c) {
                    return false;
                }
            }
            CharacterCorrectness::IncorrectPosition(c) => {
                let mut foundchar = false;
                for (idx, character) in word.chars().enumerate() {
                    if idx == correctness.pos && character == c {
                        return false;
                    } else if character == c {
                        foundchar = true;
                    }
                }
                if !foundchar {
                    return false;
                }
            }
            CharacterCorrectness::CorrectPosition(c) => {
                if c != word.chars().nth(correctness.pos).unwrap() {
                    return false;
                }
            }
        }
    }
    true
}

/// Generates the weights for a particular dictionary based on the weights from a file.
pub(crate) fn generate_dict_weights_map<'a>(
    frequency_map: Option<&'a FxHashMap<String, f64>>,
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

#[cfg(test)]
mod test {
    use crate::word_stats::{is_valid_guess, CharacterCorrectness, CharacterRENAMEME};

    #[test]
    fn valid_guess() {
        let original_guess = "weary";
        let possible_guesses = vec!["wacko", "waift", "wails", "waist", "woman", "watch"];
        let impossible_guesses = vec!["xncro", "ircpm", "dream", "crown", "weary"];
        let wg = vec![
            CharacterRENAMEME {
                pos: 0,
                character: CharacterCorrectness::CorrectPosition('w'),
            },
            CharacterRENAMEME {
                pos: 1,
                character: CharacterCorrectness::NotInWord('e'),
            },
            CharacterRENAMEME {
                pos: 2,
                character: CharacterCorrectness::IncorrectPosition('a'),
            },
            CharacterRENAMEME {
                pos: 3,
                character: CharacterCorrectness::NotInWord('r'),
            },
            CharacterRENAMEME {
                pos: 4,
                character: CharacterCorrectness::NotInWord('y'),
            },
        ];
        assert!(!is_valid_guess(&wg, original_guess));
        for guess in possible_guesses {
            assert!(is_valid_guess(&wg, guess));
        }
        for guess in impossible_guesses {
            assert!(!is_valid_guess(&wg, guess));
        }
        let original_guess = "slate";
        let wg = vec![
            CharacterRENAMEME {
                pos: 0,
                character: CharacterCorrectness::IncorrectPosition('s'),
            },
            CharacterRENAMEME {
                pos: 1,
                character: CharacterCorrectness::IncorrectPosition('l'),
            },
            CharacterRENAMEME {
                pos: 2,
                character: CharacterCorrectness::IncorrectPosition('a'),
            },
            CharacterRENAMEME {
                pos: 3,
                character: CharacterCorrectness::IncorrectPosition('t'),
            },
            CharacterRENAMEME {
                pos: 4,
                character: CharacterCorrectness::IncorrectPosition('e'),
            },
        ];
        assert!(!is_valid_guess(&wg, original_guess));
    }
}
