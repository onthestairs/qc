use std::collections::HashMap;

use clap::Parser;
use qc::generate::data::Word;
use qc::generate::qc::QuinianCrossword;
use qc::generate::search::find_solutions;
use qc::store::csv::get_clues;
use qc::store::word_frequencies::get_words_wiki_frequencies;
use qc::store::{ensure_results_table_exists, get_connection, insert_result_into_table};

/// Program to generate quinian crosswords
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Size of the grid
    #[clap(short, long, default_value_t = 3)]
    size: usize,

    /// Number of allowed non-surface words
    #[clap(short, long, default_value_t = 0)]
    allowed_missing_surfaces: usize,

    /// Start index
    #[clap(long, default_value_t = 0)]
    start_index: usize,

    /// min word frequency
    #[clap(long)]
    min_word_frequency: Option<u32>,
}

/// Should we include the surface/solution in the analysis
fn should_include(
    surface: &String,
    solution: &Word,
    size: usize,
    word_frequencies: &HashMap<Word, u32>,
    min_word_frequency: Option<u32>,
) -> bool {
    if surface.len() == 0 || surface.starts_with("See") || surface == "<<NO CLUE>>" {
        return false;
    }
    if solution.len() != size {
        return false;
    }
    let is_all_alpha_uppercase = solution.iter().all(|c| char::is_ascii_uppercase(&c));
    if !is_all_alpha_uppercase {
        return false;
    }
    if let Some(min_freq) = min_word_frequency {
        if *word_frequencies.get(solution).unwrap_or(&0) < min_freq {
            return false;
        }
    }
    return true;
}

fn filter_clues(
    clues: Vec<(String, Word)>,
    size: usize,
    word_frequencies: HashMap<Word, u32>,
    min_word_frequency: Option<u32>,
) -> Vec<(String, Word)> {
    return clues
        .into_iter()
        .filter(|(surface, solution)| {
            return should_include(
                surface,
                solution,
                size,
                &word_frequencies,
                min_word_frequency,
            );
        })
        .collect();
}

fn main() {
    let args = Args::parse();

    let connection = get_connection();
    ensure_results_table_exists(&connection);
    let clues = get_clues().unwrap();
    let word_frequencies = get_words_wiki_frequencies();
    // let filtered_clues = filter_clues(clues, args.size, word_frequencies, args.min_word_frequency);
    let filtered_clues = filter_clues(clues, 5, word_frequencies, args.min_word_frequency);
    let save_crossword = |crossword: &QuinianCrossword, size: usize, score: usize| {
        insert_result_into_table(&connection, crossword, size, score);
    };
    find_solutions(
        // args.size,
        5, 
        args.allowed_missing_surfaces,
        args.start_index,
        filtered_clues,
        save_crossword,
    );
}
