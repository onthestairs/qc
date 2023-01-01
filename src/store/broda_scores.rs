//! Broda scores

use std::collections::HashMap;
use std::fs::File;

use crate::generate::data::Word;

/// Make a map of word to how many times it is used on wikiepedia
pub fn get_broda_scores() -> HashMap<Word, u32> {
    let file = File::open("./data/broda-scores.txt").unwrap();
    let mut rdr = csv::ReaderBuilder::new().delimiter(b';').from_reader(file);
    let mut scores: HashMap<Word, u32> = HashMap::new();
    for result in rdr.records() {
        let record = result.unwrap();
        let word_str = record[0].to_string().to_uppercase();
        let word: Vec<char> = word_str.chars().collect();
        let score = record[1].parse().unwrap();
        scores.insert(word, score);
    }
    return scores;
}
