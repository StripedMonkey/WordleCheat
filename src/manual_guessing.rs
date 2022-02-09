use std::cmp;

use crate::{
    autosolve, filter_dictionary,
    word_stats::{
        self, sort_dictionary_frequency, sort_dictionary_location, CorrectPosition,
        IncorrectPosition,
    },
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

pub(crate) fn manual_guessing_entropy(mut dictionary: Vec<&str>) {
    let mut guess_dictionary = dictionary.clone();
    println!(
        "{numleft} remaining words in dictionary",
        numleft = dictionary.len()
    );
    loop {
        word_stats::sort_dictionary_entropy_progress(&mut guess_dictionary,&dictionary);
        let top_ans = guess_dictionary.get(..cmp::min(guess_dictionary.len(), 3)).unwrap();
        println!(
            "The top {:?} {}:\n{:?}",
            top_ans.len(),
            {
                if guess_dictionary.len() > 1 {
                    "answers remaining"
                } else {
                    "answer is"
                }
            },
            top_ans
        );
        if !(guess_dictionary.len() > 1) {
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
                _ => todo!("Die blaine"),
            }
        }

        filter_dictionary(
            &mut guess_dictionary,
            word,
            incorrect_positions,
            correct_positions,
        );

        println!(
            "{numleft} remaining word{s} in dictionary",
            numleft = guess_dictionary.len(),
            s = {
                if guess_dictionary.len() > 1 {
                    "s"
                } else {
                    ""
                }
            }
        );
    }
}

pub(crate) fn readline() -> String {
    let inp: String = text_io::read!("{}\n");
    return inp.trim().to_string();
}
