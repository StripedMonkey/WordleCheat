use std::cmp;

use crate::{
    autosolve, filter_dictionary,
    word_stats::{
        sort_dictionary_frequency, sort_dictionary_location, CharacterCorrectness,
        CharacterRENAMEME, CorrectPosition, IncorrectPosition,
    },
    wordle_data::Game,
};

pub(crate) fn find_path(mut dictionary: Vec<&str>) {
    println!("What is the answer you're looking for?");
    let answer = readline();
    let solution = autosolve(&mut dictionary, &answer).expect("Solver failed!");
    println!(
        "Managed to solve for {answer} in {length} steps:\n{path}",
        length = solution.1.len(),
        path = solution.1.join(",")
    )
}

pub(crate) fn manual_guessing(mut dictionary: Vec<&str>) {
    println!(
        "{numleft} remaining words in dictionary",
        numleft = dictionary.len()
    );
    loop {
        sort_dictionary_frequency(&mut dictionary);
        sort_dictionary_location(&mut dictionary);
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
        if dictionary.is_empty() {
            return;
        }

        println!("What word did you enter?");
        let word: String = readline();
        let mut incorrect_positions: Vec<IncorrectPosition> = Vec::new();
        let mut correct_positions: Vec<CorrectPosition> = Vec::new();
        for (i, character) in word.chars().enumerate() {
            println!("Was {character} correct (c) in the string (s) or nothing (empty)");
            match readline().as_str() {
                "c" => correct_positions.push(CorrectPosition {
                    position: i,
                    character,
                }),
                "s" => incorrect_positions.push(IncorrectPosition {
                    position: i,
                    character,
                }),
                "" => {}
                _ => todo!("Die blaine"),
            }
        }

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

pub(crate) fn manual_guessing_entropy(game: &mut Game) {
    println!("{} possible words remaining", game.remaining_words());
    loop {
        let answers = game.possible_words();
        let top_ans = answers.get(..cmp::min(game.remaining_words(), 3)).unwrap();
        println!(
            "The top {:?} {}:\n{:?}",
            top_ans.len(),
            {
                if top_ans.len() > 1 {
                    "answers remaining"
                } else {
                    "answer is"
                }
            },
            top_ans
        );

        println!("What word did you enter?");
        let word: String = readline();
        for (pos, character) in word.chars().enumerate() {
            println!("Was {character} correct (c) in the string (s) or nothing (empty)");
            match readline().as_str() {
                "c" => game.add_information(CharacterRENAMEME {
                    pos,
                    character: CharacterCorrectness::CorrectPosition(character),
                }),
                "s" => game.add_information(CharacterRENAMEME {
                    pos,
                    character: CharacterCorrectness::IncorrectPosition(character),
                }),
                "" => game.add_information(CharacterRENAMEME {
                    pos,
                    character: CharacterCorrectness::NotInWord(character),
                }),
                _ => println!("Skipping character..."),
            }
        }

        println!("Evaluating word data");
        // game.add_guess(&word);
        let guess_results = game.evaluate_information(&word);
        println!("Guessing {word} had an estimated entropy of {estimated},\nbut actually had an entropy of {actual}",
            word=guess_results.guess,
            estimated=guess_results.estimated_entropy,
            actual=guess_results.actual_entropy
        );
        println!(
            "There were a total of {} results",
            guess_results.possible_answers.unwrap().len()
        );

        println!("{} Remaining", game.remaining_words());
    }
}

pub(crate) fn readline() -> String {
    let inp: String = text_io::read!("{}\n");
    return inp.trim().to_string();
}
