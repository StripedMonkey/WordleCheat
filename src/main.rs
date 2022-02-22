#![feature(drain_filter)]
#[macro_use]
extern crate approx;

mod file_operations;
mod information_theory;
mod manual_guessing;
mod word_stats;
mod wordle_data;

use crate::{manual_guessing::readline, word_stats::generate_dict_weights_map};
use word_stats::{
    sort_dictionary_frequency, sort_dictionary_location, CorrectPosition, GuessResult,
    IncorrectPosition,
};

enum GuessingStrategy {
    Entropy,
    Positional,
    CharacterFrequency,
}

fn main() {
    let dictionary: Vec<&str> = include_str!("../Dictionary3b1bValidAnswers.txt")
        .split('\n')
        .collect();
    let weight_map = generate_dict_weights_map(None, &dictionary);
    // print!("How many letters is the wordle? ");
    // io::stdout().flush().unwrap();
    // let word_len: usize = readline();
    // dictionary.drain_filter(|f| f.len() != word_len);


    println!("Generate Guess Data (G), Use entropy approach (e) or heurisitc approach? (h)");
    match readline().to_ascii_lowercase().as_str() {
        "e" => {
            let mut game = wordle_data::Game::new(Some(weight_map), Some(&dictionary),"PermutationMap.bin");
            manual_guessing::manual_guessing(&mut game, GuessingStrategy::Entropy)
        },
        "h" => {
            let mut game = wordle_data::Game::new(Some(weight_map), Some(&dictionary),"PermutationMap.bin");
            println!("Solve puzzle (s) or find path (p)?");
            match readline().as_str() {
                "s" => manual_guessing::manual_guessing(&mut game, GuessingStrategy::Positional),
                "p" => manual_guessing::find_path(dictionary),
                _ => println!("Goodbye"),
            }
        }
        "g" => file_operations::generate_permutation_file("FrequencyMap.bin", "PermutationMap.bin"),
        _ => println!("Nope"),
    }
}

fn filter_dictionary(
    dictionary: &mut Vec<&str>,
    guessed_word: String,
    incorrect_positions: Vec<IncorrectPosition>,
    correct_positions: Vec<CorrectPosition>,
) {
    let mut x: String = incorrect_positions.iter().map(|c| c.character).collect();
    x.extend(correct_positions.iter().map(|p| p.character));
    dictionary.drain_filter(|f| {
        // Remove words with characters we know aren't in the word
        for character in guessed_word.chars() {
            if !(x.contains(character)) && f.contains(character) {
                return true;
            }
        }

        // Remove words that don't have characters we know are in the word
        for character in x.chars() {
            if !f.contains(character) {
                return true;
            }
        }

        // Remove words without the letters in their correct positions
        for correct in &correct_positions {
            if f.chars().nth(correct.position) != Some(correct.character) {
                return true;
            }
        }

        // Remove words with with a character in the incorrect position
        for character in &incorrect_positions {
            if Some(character.character) == f.chars().nth(character.position) {
                return true;
            }
        }
        false
    });
}

fn find_correct_positions(guess: &str, answer: &str) -> Vec<CorrectPosition> {
    let mut correct: Vec<CorrectPosition> = Vec::new();
    for (i, (guess, answer)) in guess.chars().zip(answer.chars()).enumerate() {
        if guess == answer {
            correct.push(CorrectPosition {
                position: i,
                character: answer,
            })
        }
    }
    correct
}

fn find_incorrect_characters(guess: &str, answer: &str) -> Vec<IncorrectPosition> {
    let mut correct: Vec<IncorrectPosition> = Vec::new();
    for (i, (guess_c, answer_c)) in guess.chars().zip(answer.chars()).enumerate() {
        if guess_c != answer_c && answer.contains(guess_c) {
            correct.push(IncorrectPosition {
                position: i,
                character: guess_c,
            })
        }
    }
    correct
}
fn autosolve<'dict, 'answer>(
    dictionary: &mut Vec<&'dict str>,
    answer: &'answer str,
) -> Result<GuessResult<'dict, 'answer>, GuessResult<'dict, 'answer>> {
    let mut path = Vec::new();
    loop {
        sort_dictionary_frequency(dictionary);
        sort_dictionary_location(dictionary);
        let guess = match dictionary.get(0) {
            Some(i) => {
                path.push(*i);
                *i
            }
            None => {
                println!("Managed to run out of dictionary to search while looking for {answer}");
                println!("Possibly probably not a word found in the dictionary?");
                return Err(GuessResult { answer, path });
            }
        };
        if guess == answer {
            return Ok(GuessResult { answer, path });
        }
        filter_dictionary(
            dictionary,
            guess.to_string(),
            find_incorrect_characters(guess, answer),
            find_correct_positions(guess, answer),
        );
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;

    use crate::{
        autosolve,
        word_stats::{count_unique_positions, GuessResult},
    };

    fn ans_solver<'dict, 'answer>(
        mut dictionary: Vec<&'dict str>,
        answer_word: &'answer str,
    ) -> GuessResult<'dict, 'answer> {
        let ans = autosolve(&mut dictionary, &answer_word).expect("Autosolving failed!");
        assert_eq!(ans.answer, answer_word);
        ans
    }

    #[test]
    #[ignore]
    fn strategy_length() {
        let working_dictionary: Vec<&str> =
            include_str!("../Dictionary2.txt").split("\n").collect();

        let mut guesses: Vec<GuessResult> = Vec::new();
        let solver = |&ans| ans_solver(working_dictionary.clone(), ans);
        use rayon::prelude::*;
        working_dictionary
            .par_iter()
            .map(solver)
            .collect_into_vec(&mut guesses);

        write_results(guesses, "test.csv")
    }

    fn write_results(guesses: Vec<GuessResult>, filename: &str) {
        use std::io::Write as IoWrite;
        let mut f = File::create(filename).expect("Unable to create file");
        writeln!(&mut f, "Answer,Path Length,Path").expect("Failed to write to file!");
        for guess in guesses {
            writeln!(&mut f, "{:?}", (guess.answer, guess.path)).expect("Failed to write to file!");
        }
    }
    #[test]
    fn autosolve_eater() {
        let mut working_dictionary: Vec<&str> = include_str!("../DictionaryOnlyAnswers.txt")
            .split("\n")
            .collect();
        let ans = autosolve(&mut working_dictionary, "eater").expect("Autosolving failed!");
        println!("{}", ans.path.len());
    }

    #[test]
    fn get_frequency() {
        let working_dictionary: Vec<&str> = include_str!("../DictionaryOnlyAnswers.txt")
            .split("\n")
            .collect();
        let position = count_unique_positions(&working_dictionary);
        let mut x: Vec<(&char, &Vec<usize>)> = position.iter().collect();
        x.sort_by_key(|f| f.0);
        x.iter()
            .for_each(|(character, positions)| println!("{character}: {positions:?}"));
    }
}
