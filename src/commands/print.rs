//! Allow the printing of commands

use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;

use clap::Parser;

use crate::generate::data::Word;
use crate::generate::grid::get_all_words;
use crate::generate::qc::print_qc;
use crate::generate::qc::QuinianCrossword;
use crate::store::get_connection;
use crate::store::get_results;
use crate::store::word_frequencies::get_words_wiki_frequencies;

#[derive(clap::ArgEnum, Clone)]
enum CrosswordType {
    Dense3,
    Dense4,
    Dense5,
    Alternating5,
    Alternating6,
    Alternating7,
}

/// Program to print quinian crosswords
#[derive(Parser)]
pub struct Args {
    /// Crossword type
    #[clap(arg_enum)]
    crossword_type: CrosswordType,

    /// Number of allowed non-surface words
    #[clap(short, long, default_value_t = 0)]
    allowed_missing_surfaces: usize,

    /// Max
    #[clap(short, long, default_value_t = 1_000_000)]
    max: usize,

    /// Only allow common words
    #[clap(short, long)]
    used_on_wiki_n_times: Option<u32>,

    /// Exclude banned words
    #[clap(long)]
    exclude_banned_words: bool,
}

fn get_all_used_words(solution: &QuinianCrossword) -> HashSet<Word> {
    let mut words = get_all_words(&solution.grid1);
    let ws2 = get_all_words(&solution.grid2);
    words.extend(ws2);
    let words_set: HashSet<Word> = words.iter().cloned().collect();
    return words_set;
}

fn get_banned_words() -> HashSet<Word> {
    let file = File::open("./data/banned_words.txt").unwrap();
    let reader = BufReader::new(file);
    let mut words = HashSet::new();
    for line in reader.lines() {
        let line = line.unwrap(); // Ignore errors.
        let word: Vec<char> = line.chars().collect();
        words.insert(word);
    }
    return words;
}

struct FilterOptions {
    used_on_wiki_n_times: Option<u32>,
    exclude_banned_words: bool,
}

fn is_good_solution(
    filter_options: &FilterOptions,
    common_words: &HashMap<Word, u32>,
    banned_words: &HashSet<Word>,
    solution: &QuinianCrossword,
) -> bool
where
{
    let used_words = get_all_used_words(solution);
    if let Some(min_freq) = filter_options.used_on_wiki_n_times {
        if !all_words_are_common(&common_words, min_freq, &used_words) {
            return false;
        }
    }
    if filter_options.exclude_banned_words {
        if any_banned_words_used(&banned_words, &used_words) {
            return false;
        }
    }
    return true;
}

fn any_banned_words_used(banned_words: &HashSet<Word>, used_words: &HashSet<Word>) -> bool {
    return used_words.iter().any(|w| {
        return banned_words.contains(w);
    });
}

fn all_words_are_common(
    common_words: &HashMap<Word, u32>,
    min_freq: u32,
    used_words: &HashSet<Word>,
) -> bool {
    let all_common = used_words
        .iter()
        .all(|w| *common_words.get(w).unwrap_or(&0) >= min_freq);
    return all_common;
}

fn show_crossword_type(crossword_type: CrosswordType) -> String {
    return match crossword_type {
        CrosswordType::Dense3 => "dense3".to_string(),
        CrosswordType::Dense4 => "dense4".to_string(),
        CrosswordType::Dense5 => "dense5".to_string(),
        CrosswordType::Alternating5 => "alternating5".to_string(),
        CrosswordType::Alternating6 => "alternating6".to_string(),
        CrosswordType::Alternating7 => "alternating7".to_string(),
    };
}

/// Print some QCs
pub fn run(args: Args) {
    let connection = get_connection();
    let solutions = get_results(
        &connection,
        show_crossword_type(args.crossword_type),
        args.allowed_missing_surfaces,
        args.max,
    )
    .unwrap();
    let words_wiki_frequencies = get_words_wiki_frequencies();
    let banned_words = get_banned_words();
    let filter_options = FilterOptions {
        used_on_wiki_n_times: args.used_on_wiki_n_times,
        exclude_banned_words: args.exclude_banned_words,
    };
    for (solution, minimum_broda_score) in solutions {
        if is_good_solution(
            &filter_options,
            &words_wiki_frequencies,
            &banned_words,
            &solution,
        ) {
            println!("{}", &minimum_broda_score);
            print_qc(&solution);
        }
    }
}
