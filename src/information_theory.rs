
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::word_stats::{is_valid_guess, CharacterCorrectness, WordGuess, WORDLESIZE};

fn generate_possibilities(word: &str) -> Vec<WordGuess> {
    let mut possibilities = Vec::new();
    let mut it = word.chars();
    for possibility in get_character_possibilities(it.next().unwrap()) {
        possibilities.push(vec![possibility]);
    }
    while let Some(character) = it.next() {
        let mut new_possibilities: Vec<WordGuess> = Vec::new();
        for correctness in get_character_possibilities(character) {
            let mut new = Vec::new();
            for p in &possibilities {
                let mut u = p.clone();
                u.push(correctness.clone());
                new.push(u);
            }
            new_possibilities.extend(new);
        }
        possibilities = new_possibilities;
    }

    for possibility in &possibilities {
        assert_eq!(possibility.len(), word.chars().count());
    }
    assert_eq!(possibilities.len(), 243);
    possibilities
}

fn get_character_possibilities(character: char) -> Vec<CharacterCorrectness> {
    vec![
        CharacterCorrectness::IncorrectPosition(character),
        CharacterCorrectness::CorrectPosition(character),
        CharacterCorrectness::NotInWord(character),
    ]
}

pub(crate) fn calculate_expected_entropy(guess: &str, dictionary: &Vec<&str>) -> f64 {
    calculate_entropy_distribution(guess, dictionary)
        .iter()
        .sum::<f64>()
}

fn calculate_entropy_distribution(guess: &str, dictionary: &Vec<&str>) -> Vec<f64> {
    generate_possibilities(guess)
        .iter()
        .map(|pattern| {
            let probability = calculate_pattern_probability(pattern, dictionary);
            let entropy = calculate_pattern_entropy(probability);
            let information = probability * entropy;
            if information.is_nan() {
                return 0.0;
            }
            information
        })
        .collect()
}

fn calculate_pattern_entropy(probability: f64) -> f64 {
    (1.0 / probability).log2()
}

fn calculate_pattern_probability(validation: &WordGuess, dictionary: &Vec<&str>) -> f64 {
    let prob = count_pattern_possibilities(validation, dictionary) as f64 / dictionary.len() as f64;
    prob
}

fn count_pattern_possibilities(validation: &WordGuess, dictionary: &Vec<&str>) -> usize {
    dictionary
        .par_iter()
        .filter(|s| is_valid_guess(validation, s))
        .count()
}

fn get_correctness(guess: &str, answer: &str) -> WordGuess {
    let mut result: Vec<CharacterCorrectness> = Vec::new();
    for (answer_character, guess_character) in answer.chars().zip(guess.chars()) {
        if answer_character == guess_character {
            result.push(CharacterCorrectness::CorrectPosition(answer_character))
        } else if guess.contains(answer_character) {
            result.push(CharacterCorrectness::IncorrectPosition(guess_character))
        } else {
            result.push(CharacterCorrectness::NotInWord(guess_character))
        }
    }
    assert_eq!(result.len(), WORDLESIZE);
    result
}

#[cfg(test)]
mod test {
    const CURRENT_DICTIONARY: &str = "../Dictionary3b1bValidAnswers.txt";
    use indicatif::{ParallelProgressIterator, ProgressBar, ProgressStyle};
    use ordered_float::NotNan;
    use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

    use crate::filter_dictionary;
    use crate::information_theory::count_pattern_possibilities;
    use crate::word_stats::{CharacterCorrectness::*, WORDLESIZE};

    use super::{calculate_expected_entropy, generate_possibilities};

    #[test]
    fn slate_entropy() {
        // Using the 3b1b Dictionary this is an exact value
        let working_dictionary: Vec<&str> = include_str!("../Dictionary3b1bValidAnswers.txt")
            .split("\n")
            .collect();
        assert_relative_eq!(
            5.871215373158421,
            calculate_expected_entropy("slate", &working_dictionary)
        );
    }

    #[test]
    fn weary_count() {
        let mut working_dictionary: Vec<&str> = include_str!("../Dictionary3b1bValidAnswers.txt")
            .split("\n")
            .collect();

        let count = count_pattern_possibilities(
            &vec![
                CorrectPosition('w'),
                NotInWord('e'),
                IncorrectPosition('a'),
                NotInWord('r'),
                NotInWord('y'),
            ],
            &working_dictionary,
        );

        filter_dictionary(
            &mut working_dictionary,
            "weary".to_string(),
            vec![crate::word_stats::IncorrectPosition {
                pos: 2,
                character: 'a',
            }],
            vec![crate::word_stats::CorrectPosition {
                pos: 0,
                character: 'w',
            }],
        );
        assert_eq!(working_dictionary.len(), count);
    }

    #[test]
    fn entropy_calculations() {
        let working_dictionary: Vec<&str> = include_str!("../Dictionary3b1bValidAnswers.txt")
            .split("\n")
            .collect();
        let temp_dictionary = working_dictionary.clone();
        let mut xx: Vec<(&str, f64)> = working_dictionary
            .par_iter()
            .map(|s| (*s, calculate_expected_entropy(s, &temp_dictionary)))
            .collect();
        xx.sort_by_key(|s| NotNan::new(s.1).unwrap());
        write_results(xx, "SortedEntropy");
    }

    fn write_results(data: Vec<(&str, f64)>, filename: &str) {
        use std::fs::File;
        use std::io::Write as IoWrite;
        let mut f = File::create(filename).expect("Unable to create file");
        for line in data {
            writeln!(&mut f, "{:?}", line).expect("Failed to write to file!");
        }
    }

    #[test]
    fn possibility_count() {
        let word = "slate";
        let x = generate_possibilities(word);
        assert_eq!(x.len() as f64, 3.0_f64.powi(word.len() as i32))
    }

    #[test]
    fn maximum_entropy() {
        // For a given dictionary size the maximum entropy value possible is log_2(a^n)
        // Where n is the length of the word and a is the number of states a value can be in
        let mut working_dictionary: Vec<&str> = include_str!("../Dictionary3b1bValidAnswers.txt")
            .split("\n")
            .collect();
        let reference_dictionary = working_dictionary.clone();
        let max_entropy = 3.0_f64.powi(WORDLESIZE as i32);
        for word in reference_dictionary {}
    }
}
