//! Word frequencies

use std::{collections::HashMap, fs::File};

use crate::generate::data::Word;

/// Make a map of word to how many times it is used on wikiepedia
pub fn get_words_wiki_frequencies() -> HashMap<Word, u32> {
    let file = File::open("./data/en_words_1_3-5.txt").unwrap();
    let mut rdr = csv::ReaderBuilder::new().delimiter(b' ').from_reader(file);
    let mut word_freqs: HashMap<Word, u32> = HashMap::new();
    for result in rdr.records() {
        let record = result.unwrap();
        let word_str = record[0].to_string().to_uppercase();
        let word: Vec<char> = word_str.chars().collect();
        let frequency = record[2].parse().unwrap();
        word_freqs.insert(word, frequency);
    }
    return word_freqs;
}
