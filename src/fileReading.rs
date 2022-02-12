use std::fs::read_to_string;

use rustc_hash::FxHashMap;

// Quick and dirty reading of frequency tables
fn read_file(filename: &str, word_size: usize) -> FxHashMap<&str, f64> {
    let file = read_to_string(filename).unwrap();
    let temp_map: FxHashMap<&str, f64> = file
        .lines()
        .map(|line| {
            let split = line.split_ascii_whitespace();
            (
                split.next().unwrap(),
                split.next().unwrap().parse::<f64>().unwrap(),
            )
        })
        .collect();
    let sum: f64 = temp_map.values().sum();
    temp_map
        .into_iter()
        .map(|(s, i)| (s, i as f64 / sum))
        .collect::<FxHashMap<&str, f64>>()
}
