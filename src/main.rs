#![feature(drain_filter)]

use std::{cmp, collections::HashMap};
use text_io;

fn main() {
    let dictionary: Vec<&str> = include_str!("../DictionaryOnlyAnswers.txt")
        .split("\n")
        .collect();
    // print!("How many letters is the wordle? ");
    // io::stdout().flush().unwrap();
    // let word_len: usize = readline();
    // dictionary.drain_filter(|f| f.len() != word_len);

    println!("Solve puzzle (s) or find path (p)?");
    match readline().as_str() {
        "s" => manual_guessing(dictionary),
        "p" => find_path(dictionary),
        _ => println!("Goodbye"),
    }
}

struct CharCounter {
    pos: usize,
    character: char,
}
type IncorrectPosition = CharCounter;
type CorrectPosition = CharCounter;

fn find_path(mut dictionary: Vec<&str>) {
    println!("What is the answer you're looking for?");
    let answer = readline();
    let solution = autosolve(&mut dictionary, &answer).expect("Solver failed!");
    println!(
        "Managed to solve for {answer} in {length} steps:\n{path}",
        length = solution.1.len(),
        path = solution.1.join(",")
    )
}
fn manual_guessing(mut dictionary: Vec<&str>) {
    println!(
        "{numleft} remaining words in dictionary",
        numleft = dictionary.len()
    );
    loop {
        sort_dictionary_frequency(&mut dictionary);
        let top_ans = dictionary.get(..cmp::min(dictionary.len(), 3)).unwrap();
        println!(
            "The top {:?} {}:\n{:?}",
            top_ans.len(),
            {
                if dictionary.len() > 1 {
                    "answers remaining"
                } else {
                    "answer is"
                }
            },
            top_ans
        );
        if !(dictionary.len() > 1) {
            return;
        }

        println!("What word did you enter?");
        let word: String = readline();
        let mut incorrect_positions: Vec<IncorrectPosition> = Vec::new();
        let mut correct_positions: Vec<CorrectPosition> = Vec::new();
        for (i, character) in word.chars().enumerate() {
            println!("Was {character} correct (c) in the string (s) or nothing (empty)");
            match readline().as_str() {
                "c" => correct_positions.push(CorrectPosition { pos: i, character }),
                "s" => incorrect_positions.push(IncorrectPosition { pos: i, character }),
                "" => {}
                _ => todo!(),
            }
        }

        sort_dictionary_frequency(&mut dictionary);
        sort_dictionary_location(&mut dictionary);
        filter_dictionary(
            &mut dictionary,
            word,
            incorrect_positions,
            correct_positions,
        );

        println!(
            "{numleft} remaining word{s} in dictionary",
            numleft = dictionary.len(),
            s = {
                if dictionary.len() > 1 {
                    "s"
                } else {
                    ""
                }
            }
        );
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
            if !(x.contains(character)) {
                if f.contains(character) {
                    return true;
                }
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
            if f.chars().nth(correct.pos) != Some(correct.character) {
                return true;
            }
        }

        // Remove words with with a character in the incorrect position
        for character in &incorrect_positions {
            if Some(character.character) == f.chars().nth(character.pos) {
                return true;
            }
        }
        return false;
    });
}

fn count_unique_chars(s: &str) -> usize {
    let mut uniq_characters: Vec<char> = Vec::new();
    for character in s.chars() {
        if !uniq_characters.contains(&character) {
            uniq_characters.push(character);
        }
    }
    uniq_characters.len()
}

fn evaulate_uniqueness(unique_characters: &Vec<CharCounter>, word: &str) -> usize {
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

fn get_character_frequencies(dictionary: &Vec<&str>) -> Vec<CharCounter> {
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
fn sort_dictionary_frequency(dictionary: &mut Vec<&str>) {
    // Sort by the frequency of unique characters in each word
    let unique_characters = get_character_frequencies(&dictionary);
    dictionary.sort_by_cached_key(|word| evaulate_uniqueness(&unique_characters, word));
    dictionary.reverse();
}

fn count_location(positions: &HashMap<char, Vec<usize>>, word: &str) -> usize {
    let mut value: usize = 0;
    for (i, character) in word.chars().enumerate() {
        value += positions.get(&character).expect("Couldn't find character!")[i];
    }
    value
}

fn count_unique_positions(dictionary: &Vec<&str>) -> HashMap<char, Vec<usize>> {
    let mut unique_positions: HashMap<char, Vec<usize>> = HashMap::new(); // TODO: Make this not fixed length
    for word in &*dictionary {
        for (i, character) in word.chars().enumerate() {
            match unique_positions.get_mut(&character) {
                Some(arr) => arr[i] += 1,
                None => {
                    let mut positions = vec![0;5];
                    positions[i] += 1;
                    unique_positions.insert(character, positions);
                }
            }
        }
    }
    unique_positions
}

fn sort_dictionary_location(dictionary: &mut Vec<&str>) {
    let unique_positions = count_unique_positions(&dictionary);
    dictionary.sort_by_cached_key(|a| count_location(&unique_positions, a));
    dictionary.reverse();
}

fn readline() -> String {
    let inp: String = text_io::read!("{}\n");
    return inp.trim().to_string();
}

fn find_correct_positions(guess: &str, answer: &str) -> Vec<CorrectPosition> {
    let mut correct: Vec<CorrectPosition> = Vec::new();
    for (i, (guess, answer)) in guess.chars().zip(answer.chars()).enumerate() {
        if guess == answer {
            correct.push(CorrectPosition {
                pos: i,
                character: answer,
            })
        }
    }
    correct
}

fn find_correct_characters(guess: &str, answer: &str) -> Vec<IncorrectPosition> {
    let mut correct: Vec<IncorrectPosition> = Vec::new();
    for (i, (guess_c, answer_c)) in guess.chars().zip(answer.chars()).enumerate() {
        if guess_c != answer_c {
            if answer.contains(guess_c) {
                correct.push(IncorrectPosition {
                    pos: i,
                    character: guess_c,
                })
            }
        }
    }
    correct
}
fn autosolve<'a, 'b>(
    dictionary: &mut Vec<&'a str>,
    answer: &'b str,
) -> Result<(&'a str, Vec<&'a str>), (&'b str, Vec<&'a str>)> {
    let mut guesses = Vec::new();
    loop {
        sort_dictionary_frequency(dictionary);
        sort_dictionary_location(dictionary);
        let guess = match dictionary.get(0) {
            Some(i) => {
                guesses.push(*i);
                *i
            }
            None => {
                println!("Managed to run out of dictionary to search while looking for {answer}");
                println!("Possibly probably not a word found in the dictionary?");
                return Err((answer, guesses));
            }
        };
        if guess == answer {
            return Ok((guess, guesses));
        }
        filter_dictionary(
            dictionary,
            guess.to_string(),
            find_correct_characters(guess, answer),
            find_correct_positions(guess, answer),
        );
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;

    use crate::{autosolve, count_unique_positions};

    #[test]
    fn strategy_length() {
        let working_dictionary: Vec<&str> = include_str!("../DictionaryOnlyAnswers.txt")
            .split("\n")
            .collect();

        let mut guesses: Vec<(&str, Vec<&str>)> = Vec::new();
        let solver = |answer_word: &&str| {
            let mut dictionary = working_dictionary.clone();
            let ans = autosolve(&mut dictionary, &answer_word).expect("Autosolving failed!");
            assert_eq!(ans.0, *answer_word);
            ans
        };
        use rayon::prelude::*;
        working_dictionary
            .par_iter()
            .map(solver)
            .collect_into_vec(&mut guesses);

        write_results(guesses, "test.csv")
    }

    fn write_results(guesses: Vec<(&str, Vec<&str>)>, filename: &str) {
        use std::io::Write as IoWrite;
        let mut f = File::create(filename).expect("Unable to create file");
        writeln!(&mut f, "Answer,Path Length,Path").expect("Failed to write to file!");
        for (answer, path) in guesses {
            writeln!(
                &mut f,
                "{:?}",(answer,path)
            )
            .expect("Failed to write to file!");
        }
    }
    #[test]
    fn autosolve_eater() {
        let mut working_dictionary: Vec<&str> = include_str!("../DictionaryOnlyAnswers.txt")
            .split("\n")
            .collect();
        let ans = autosolve(&mut working_dictionary, "eater").expect("Autosolving failed!");
        println!("{}", ans.1.len());
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
