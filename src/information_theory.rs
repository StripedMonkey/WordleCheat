use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use rustc_hash::FxHashMap;

use crate::{
    file_operations::{SingleGuess, WordPossibilities},
    word_stats::{is_valid_guess, CharacterCorrectness, CharacterRENAMEME, WordGuess, WORDLESIZE},
};

/// Generates possible wordle combinations from the given set. I.E all incorrect, all correct, etc.
pub(crate) fn generate_possibilities(word: &str) -> WordPossibilities {
    let mut possibilities: Vec<SingleGuess> = Vec::new();
    let mut it = word.chars().enumerate();
    for possibility in get_character_possibilities(it.next().unwrap().1) {
        possibilities.push(
            vec![CharacterRENAMEME {
                pos: 0,
                character: possibility,
            }]
            .into(),
        );
    }
    for (pos, character) in it {
        let mut new_possibilities: Vec<SingleGuess> = Vec::new();
        for correctness in get_character_possibilities(character) {
            let mut new: Vec<SingleGuess> = Vec::new();
            for p in &possibilities {
                let mut u = p.clone();
                u.guess.push(CharacterRENAMEME {
                    pos,
                    character: correctness.clone(),
                });
                new.push(u);
            }
            new_possibilities.extend(new);
        }
        possibilities = new_possibilities;
    }

    for possibility in &possibilities {
        assert_eq!(possibility.guess.len(), word.chars().count());
    }
    assert_eq!(possibilities.len(), 3_usize.pow(word.len() as u32));
    WordPossibilities {
        guesses: possibilities,
    }
}

fn get_character_possibilities(character: char) -> Vec<CharacterCorrectness> {
    vec![
        CharacterCorrectness::IncorrectPosition(character),
        CharacterCorrectness::CorrectPosition(character),
        CharacterCorrectness::NotInWord(character),
    ]
}

pub(crate) fn calculate_expected_entropy(
    guess: &str,
    dictionary: &Vec<&str>,
    weights: &FxHashMap<&str, f64>,
) -> f64 {
    calculate_entropy_distribution(guess, dictionary, weights)
        .iter()
        .sum::<f64>()
}

pub(crate) fn calculate_entropy_distribution(
    guess: &str,
    dictionary: &Vec<&str>,
    weights: &FxHashMap<&str, f64>,
) -> Vec<f64> {
    generate_possibilities(guess)
        .guesses
        .iter()
        .map(|pattern| {
            let probability = calculate_pattern_probability(pattern, dictionary, &weights);
            let entropy = calculate_pattern_entropy(probability);
            let information = probability * entropy;
            if information.is_nan() {
                return 0.0;
            }
            information
        })
        .collect()
}

pub fn calculate_pattern_entropy(probability: f64) -> f64 {
    (1.0 / probability).log2()
}

fn calculate_pattern_probability(
    validation: &SingleGuess,
    dictionary: &Vec<&str>,
    weights: &FxHashMap<&str, f64>,
) -> f64 {
    let prob = get_pattern_words(validation, dictionary)
        .iter()
        .map(|f| weights.get(*f).unwrap_or(&0_f64))
        .sum::<f64>();
    // TODO: Come up with a slightly more reasonable way to do this.
    prob / weights.values().sum::<f64>()
}

fn get_pattern_words<'a>(validation: &SingleGuess, dictionary: &Vec<&'a str>) -> Vec<&'a str> {
    let lst = dictionary
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

fn count_valid_patterns(validation: &WordGuess, dictionary: &Vec<&str>) -> usize {
    dictionary
        .par_iter()
        .filter(|s| is_valid_guess(validation, s))
        .count()
}

fn get_correctness(guess: &str, answer: &str) -> WordGuess {
    let mut result: Vec<CharacterRENAMEME> = Vec::new();
    for (i, (answer_character, guess_character)) in answer.chars().zip(guess.chars()).enumerate() {
        if answer_character == guess_character {
            result.push(CharacterRENAMEME {
                pos: i,
                character: CharacterCorrectness::CorrectPosition(answer_character),
            })
        } else if guess.contains(answer_character) {
            result.push(CharacterRENAMEME {
                pos: i,
                character: CharacterCorrectness::IncorrectPosition(guess_character),
            })
        } else {
            result.push(CharacterRENAMEME {
                pos: i,
                character: CharacterCorrectness::NotInWord(guess_character),
            })
        }
    }
    assert_eq!(result.len(), WORDLESIZE);
    result
}

#[cfg(test)]
mod test {
    use ordered_float::NotNan;
    use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

    use crate::file_operations::read_frequency_file;
    use crate::filter_dictionary;
    use crate::information_theory::count_valid_patterns;
    use crate::word_stats::{
        generate_dict_weights_map, CharacterCorrectness::*, CharacterRENAMEME,
    };

    use super::{calculate_expected_entropy, generate_possibilities};

    #[test]
    fn slate_entropy() {
        // Using the 3b1b Dictionary this is an exact value
        let working_dictionary: Vec<&str> = include_str!("../Dictionary3b1bValidAnswers.txt")
            .split("\n")
            .collect();
        assert_relative_eq!(
            5.871215373158421,
            calculate_expected_entropy(
                "slate",
                &working_dictionary,
                &generate_dict_weights_map(None, &working_dictionary)
            )
        );
    }

    #[test]
    fn weary_count() {
        let mut working_dictionary: Vec<&str> = include_str!("../Dictionary3b1bValidAnswers.txt")
            .split("\n")
            .collect();

        let count = count_valid_patterns(
            &vec![
                CharacterRENAMEME {
                    pos: 0,
                    character: CorrectPosition('w'),
                },
                CharacterRENAMEME {
                    pos: 1,
                    character: NotInWord('e'),
                },
                CharacterRENAMEME {
                    pos: 2,
                    character: IncorrectPosition('a'),
                },
                CharacterRENAMEME {
                    pos: 3,
                    character: NotInWord('r'),
                },
                CharacterRENAMEME {
                    pos: 4,
                    character: NotInWord('y'),
                },
            ],
            &working_dictionary,
        );

        filter_dictionary(
            &mut working_dictionary,
            "weary".to_string(),
            vec![crate::word_stats::IncorrectPosition {
                position: 2,
                character: 'a',
            }],
            vec![crate::word_stats::CorrectPosition {
                position: 0,
                character: 'w',
            }],
        );
        assert_eq!(working_dictionary.len(), count);
    }

    #[test]
    #[ignore]
    fn entropy_calculations() {
        let working_dictionary: Vec<&str> = include_str!("../Dictionary3b1bValidAnswers.txt")
            .split("\n")
            .collect();
        let temp_dictionary = working_dictionary.clone();
        let freq_map = &read_frequency_file("count_1w.txt");
        let mut xx: Vec<(&str, f64)> = working_dictionary
            .par_iter()
            .map(|s| {
                (
                    *s,
                    calculate_expected_entropy(
                        s,
                        &temp_dictionary,
                        &generate_dict_weights_map(Some(freq_map), &working_dictionary),
                    ),
                )
            })
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
        assert_eq!(x.guesses.len() as f64, 3.0_f64.powi(word.len() as i32))
    }

    // #[test]
    // fn maximum_entropy() {
    //     // For a given dictionary size the maximum entropy value possible is log_2(a^n)
    //     // Where n is the length of the word and a is the number of states a value can be in
    //     let mut working_dictionary: Vec<&str> = include_str!("../Dictionary3b1bValidAnswers.txt")
    //         .split("\n")
    //         .collect();
    //     let reference_dictionary = working_dictionary.clone();
    //     let max_entropy = 3.0_f64.powi(WORDLESIZE as i32);
    //     for word in reference_dictionary {}
    // }
}
