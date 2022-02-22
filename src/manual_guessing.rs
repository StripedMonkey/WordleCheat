use std::cmp;

use crate::{
    autosolve,
    word_stats::{CharacterCorrectness, CharacterRENAMEME},
    wordle_data::Game,
    GuessingStrategy,
};

pub(crate) fn find_path<'dict>(mut dictionary: Vec<&'dict str>) {
    println!("What is the answer you're looking for?");
    let answer = readline();
    let solution = autosolve(&mut dictionary, &answer).expect("Solver failed!");
    println!(
        "Managed to solve for {answer} in {length} steps:\n{path}",
        length = solution.path.len(),
        path = solution.path.join(",")
    )
}

pub(crate) fn manual_guessing(game: &mut Game, strategy: GuessingStrategy) {
    println!("{} possible words remaining", game.remaining_words());
    loop {
        println!("Prioritizing words...");
        match strategy {
            GuessingStrategy::Entropy => game.prioritize_entropy(),
            GuessingStrategy::Positional => game.prioritize_position(),
            GuessingStrategy::CharacterFrequency => todo!(),
        };
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
        println!("Guessing {word} had an estimated entropy of {estimated:0.4},\nbut actually had an entropy of {actual:0.4}",
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
