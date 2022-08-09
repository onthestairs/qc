use std::collections::HashMap;

use clap::Parser;
use qc::generate::data::Word;
use qc::generate::qc::QuinianCrossword;
use qc::generate::search::find_grids_with_searcher;
use qc::generate::search::searchers::alternating::Alternating;
use qc::generate::search::searchers::dense::Dense;
use qc::generate::search::searchers::Searcher;
use qc::store::csv::get_clues;
use qc::store::word_frequencies::get_words_wiki_frequencies;
use qc::store::{ensure_results_table_exists, get_connection, insert_result_into_table};

/// Program to generate quinian crosswords
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(arg_enum)]
    searcher: CrosswordType,

    /// Number of allowed non-surface words
    #[clap(short, long, default_value_t = 0)]
    allowed_missing_surfaces: usize,

    /// Start index
    #[clap(short, long, default_value_t = 0)]
    start_index: usize,

    /// min word frequency
    #[clap(long)]
    min_word_frequency: Option<u32>,
}

#[derive(clap::ArgEnum, Clone, Debug)]
enum CrosswordType {
    Dense3,
    Dense4,
    Dense5,
    Alternating5,
    Alternating6,
    Alternating7,
}

/// Should we include the surface/solution in the analysis
fn should_include(
    surface: &String,
    solution: &Word,
    word_frequencies: &HashMap<Word, u32>,
    min_word_frequency: Option<u32>,
) -> bool {
    if surface.len() == 0 || surface.starts_with("See") || surface == "<<NO CLUE>>" {
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
    word_frequencies: HashMap<Word, u32>,
    min_word_frequency: Option<u32>,
) -> Vec<(String, Word)> {
    return clues
        .into_iter()
        .filter(|(surface, solution)| {
            return should_include(surface, solution, &word_frequencies, min_word_frequency);
        })
        .collect();
}

fn main() {
    let args = Args::parse();

    let connection = get_connection();
    ensure_results_table_exists(&connection);
    let clues = get_clues().unwrap();
    let word_frequencies = get_words_wiki_frequencies();
    let filtered_clues = filter_clues(clues, word_frequencies, args.min_word_frequency);
    let save_crossword = |crossword: &QuinianCrossword, crossword_type: String, score: usize| {
        insert_result_into_table(&connection, crossword, crossword_type, score);
    };
    match args.searcher {
        CrosswordType::Dense3 => {
            let searcher = Dense::new(3, filtered_clues);
            find_grids_with_searcher(
                args.start_index,
                args.allowed_missing_surfaces,
                &searcher,
                save_crossword,
            );
        }
        CrosswordType::Dense4 => {
            let searcher = Dense::new(4, filtered_clues);
            find_grids_with_searcher(
                args.start_index,
                args.allowed_missing_surfaces,
                &searcher,
                save_crossword,
            );
        }
        CrosswordType::Dense5 => {
            let searcher = Dense::new(5, filtered_clues);
            find_grids_with_searcher(
                args.start_index,
                args.allowed_missing_surfaces,
                &searcher,
                save_crossword,
            );
        }
        CrosswordType::Alternating5 => {
            let searcher = Alternating::new(5, filtered_clues);
            find_grids_with_searcher(
                args.start_index,
                args.allowed_missing_surfaces,
                &searcher,
                save_crossword,
            );
        }
        CrosswordType::Alternating6 => {
            let searcher = Alternating::new(6, filtered_clues);
            find_grids_with_searcher(
                args.start_index,
                args.allowed_missing_surfaces,
                &searcher,
                save_crossword,
            );
        }
        CrosswordType::Alternating7 => {
            let searcher = Alternating::new(7, filtered_clues);
            find_grids_with_searcher(
                args.start_index,
                args.allowed_missing_surfaces,
                &searcher,
                save_crossword,
            );
        }
    };
}
