use std::collections::HashMap;

use indicatif::{ParallelProgressIterator, ProgressIterator};
use ordered_float::NotNan;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::information_theory::calculate_expected_entropy;

pub(crate) const WORDLESIZE: usize = 5;

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum CharacterCorrectness {
    NotInWord(char),
    IncorrectPosition(char),
    CorrectPosition(char),
}
pub(crate) struct CharCounter {
    pub pos: usize,
    pub character: char,
}
pub(crate) type WordGuess = Vec<CharacterCorrectness>;
pub(crate) type IncorrectPosition = CharCounter;
pub(crate) type CorrectPosition = CharCounter;

pub(crate) fn count_unique_chars(s: &str) -> usize {
    let mut uniq_characters: Vec<char> = Vec::new();
    for character in s.chars() {
        if !uniq_characters.contains(&character) {
            uniq_characters.push(character);
        }
    }
    uniq_characters.len()
}

pub(crate) fn evaulate_uniqueness(unique_characters: &Vec<CharCounter>, word: &str) -> usize {
    let mut used = Vec::new();
    let mut value: usize = 0;
    for character in word.chars() {
        if !used.contains(&character) {
            used.push(character);
            let idx = unique_characters
                .binary_search_by_key(&character, |f| f.character)
                .expect("Couldn't find character in list!");
            value += unique_characters[idx].pos;
        }
    }
    value
}

pub(crate) fn character_frequencies(dictionary: &Vec<&str>) -> Vec<CharCounter> {
    let mut unique_characters_list: Vec<CharCounter> = Vec::new();
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
                    unique_characters_list[i] = CharCounter {
                        // Should actually just increment this, but meh
                        character,
                        pos: unique_characters_list[i].pos + 1,
                    }
                }
                Err(i) => unique_characters_list.insert(i, CharCounter { character, pos: 1 }),
            }
        }
    }
    unique_characters_list
}
pub(crate) fn sort_dictionary_frequency(dictionary: &mut Vec<&str>) {
    // Sort by the frequency of unique characters in each word
    let unique_characters = character_frequencies(&dictionary);
    dictionary.sort_by_cached_key(|word| evaulate_uniqueness(&unique_characters, word));
    dictionary.reverse();
}

pub(crate) fn sort_dictionary_location(dictionary: &mut Vec<&str>) {
    let unique_positions = count_unique_positions(&dictionary);
    dictionary.sort_by_cached_key(|a| count_location(&unique_positions, a));
    dictionary.reverse();
}

pub(crate) fn sort_dictionary_entropy_progress(guess_dictionary: &mut Vec<&str>,filtered_dictionary: &Vec<&str>) {
    let map: HashMap<&str, f64> = guess_dictionary
        .par_iter().progress_with(
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
        .map(|s| (*s, calculate_expected_entropy(s, &filtered_dictionary)))
        .collect();
    guess_dictionary.sort_by_cached_key(|s| NotNan::new(*map.get(s).unwrap()).unwrap());
    guess_dictionary.reverse();

}

pub(crate) fn count_location(positions: &HashMap<char, Vec<usize>>, word: &str) -> usize {
    let mut value: usize = 0;
    for (i, character) in word.chars().enumerate() {
        value += positions.get(&character).expect("Couldn't find character!")[i];
    }
    value
}

pub(crate) fn count_unique_positions(dictionary: &Vec<&str>) -> HashMap<char, Vec<usize>> {
    let mut unique_positions: HashMap<char, Vec<usize>> = HashMap::new();
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

pub(crate) fn is_valid_guess(validation: &WordGuess, word: &str) -> bool {
    for (word_character, guess_character) in word.chars().zip(validation.iter()) {
        match guess_character {
            CharacterCorrectness::NotInWord(c) => {
                if word.contains(*c) {
                    return false;
                }
            }
            CharacterCorrectness::IncorrectPosition(c) => {
                if *c == word_character || !word.contains(*c) {
                    return false;
                }
            }
            CharacterCorrectness::CorrectPosition(c) => {
                if *c != word_character {
                    return false;
                }
            }
        }
    }
    return true;
}

#[cfg(test)]
mod test {
    use crate::word_stats::{is_valid_guess, CharacterCorrectness, WordGuess};

    #[test]
    fn valid_guess() {
        let original_guess = "weary";
        let possible_guesses = vec!["wacko", "waift", "wails", "waist", "woman", "watch"];
        let impossible_guesses = vec!["xncro", "ircpm", "dream", "crown", "weary"];
        let wg: WordGuess = vec![
            CharacterCorrectness::CorrectPosition('w'),
            CharacterCorrectness::NotInWord('e'),
            CharacterCorrectness::IncorrectPosition('a'),
            CharacterCorrectness::NotInWord('r'),
            CharacterCorrectness::NotInWord('y'),
        ];
        assert!(!is_valid_guess(&wg, original_guess));
        for guess in possible_guesses {
            assert!(is_valid_guess(&wg, guess));
        }
        for guess in impossible_guesses {
            assert!(!is_valid_guess(&wg, guess));
        }
        let original_guess = "slate";
        let wg: WordGuess = vec![
            CharacterCorrectness::IncorrectPosition('s'),
            CharacterCorrectness::IncorrectPosition('l'),
            CharacterCorrectness::IncorrectPosition('a'),
            CharacterCorrectness::IncorrectPosition('t'),
            CharacterCorrectness::IncorrectPosition('e'),
        ];
        assert!(!is_valid_guess(&wg, original_guess));
    }
}
