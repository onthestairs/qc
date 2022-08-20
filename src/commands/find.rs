//! Find some quinian crosswords

use std::collections::HashMap;

use clap::Parser;

use crate::generate::data::Surface;
use crate::generate::data::Word;
use crate::generate::qc::QuinianCrossword;
use crate::generate::search::find_grids_with_searcher;
use crate::generate::search::searchers::alternating::Alternating;
use crate::generate::search::searchers::dense::Dense;
use crate::generate::search::searchers::Searcher;
use crate::store::csv::get_clues;
use crate::store::ensure_results_table_exists;
use crate::store::get_connection;
use crate::store::insert_result_into_table;
use crate::store::word_frequencies::get_words_wiki_frequencies;

/// Program to generate quinian crosswords
#[derive(Parser, Debug)]
pub struct Args {
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
    // exclude any bad surfaces (just uses various heuristics for now)
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

/// Run the find command
pub fn run(args: Args) {
    let connection = get_connection();
    ensure_results_table_exists(&connection);
    let mut save_crossword =
        |crossword: &QuinianCrossword, crossword_type: String, score: usize| {
            insert_result_into_table(&connection, crossword, crossword_type, score);
        };
    let clues = get_clues().unwrap();
    find_qcs(
        clues,
        args.searcher,
        args.min_word_frequency,
        args.start_index,
        args.allowed_missing_surfaces,
        &mut save_crossword,
    );
}

fn find_qcs<F>(
    clues: Vec<(Surface, Word)>,
    searcher: CrosswordType,
    min_word_frequency: Option<u32>,
    start_index: usize,
    allowed_missing_surfaces: usize,
    on_found: &mut F,
) where
    F: FnMut(&QuinianCrossword, String, usize) -> (),
{
    let word_frequencies = get_words_wiki_frequencies();
    let filtered_clues = filter_clues(clues, word_frequencies, min_word_frequency);
    match searcher {
        CrosswordType::Dense3 => {
            let searcher = Dense::new(3, filtered_clues);
            find_grids_with_searcher(start_index, allowed_missing_surfaces, &searcher, on_found);
        }
        CrosswordType::Dense4 => {
            let searcher = Dense::new(4, filtered_clues);
            find_grids_with_searcher(start_index, allowed_missing_surfaces, &searcher, on_found);
        }
        CrosswordType::Dense5 => {
            let searcher = Dense::new(5, filtered_clues);
            find_grids_with_searcher(start_index, allowed_missing_surfaces, &searcher, on_found);
        }
        CrosswordType::Alternating5 => {
            let searcher = Alternating::new(5, filtered_clues);
            find_grids_with_searcher(start_index, allowed_missing_surfaces, &searcher, on_found);
        }
        CrosswordType::Alternating6 => {
            let searcher = Alternating::new(6, filtered_clues);
            find_grids_with_searcher(start_index, allowed_missing_surfaces, &searcher, on_found);
        }
        CrosswordType::Alternating7 => {
            let searcher = Alternating::new(7, filtered_clues);
            find_grids_with_searcher(start_index, allowed_missing_surfaces, &searcher, on_found);
        }
    };
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn simple_dense3() {
        // Construct the clues so that it should be able to
        // make this crossword (and its transpose)
        // ABC   JKL
        // DEF   MNO
        // GHI   PQR
        // 1. A1     1. D1
        // 2. A2     2. D2
        // 3. A3     3. D3
        let clues = vec![
            ("A1".to_string(), vec!['A', 'B', 'C']),
            ("A1".to_string(), vec!['J', 'K', 'L']),
            ("A2".to_string(), vec!['D', 'E', 'F']),
            ("A2".to_string(), vec!['M', 'N', 'O']),
            ("A3".to_string(), vec!['G', 'H', 'I']),
            ("A3".to_string(), vec!['P', 'Q', 'R']),
            // down clues
            ("D1".to_string(), vec!['A', 'D', 'G']),
            ("D1".to_string(), vec!['J', 'M', 'P']),
            ("D2".to_string(), vec!['B', 'E', 'H']),
            ("D2".to_string(), vec!['K', 'N', 'Q']),
            ("D3".to_string(), vec!['C', 'F', 'I']),
            ("D3".to_string(), vec!['L', 'O', 'R']),
        ];
        let mut quines: HashSet<QuinianCrossword> = HashSet::new();

        let mut store_quine =
            |crossword: &QuinianCrossword, _crossword_type: String, _score: usize| {
                quines.insert(crossword.clone());
            };
        find_qcs(clues, CrosswordType::Dense3, None, 0, 0, &mut store_quine);

        let expected_quines_list = vec![
            QuinianCrossword {
                grid1: vec![
                    vec!['A', 'D', 'G'],
                    vec!['B', 'E', 'H'],
                    vec!['C', 'F', 'I'],
                ],
                grid2: vec![
                    vec!['J', 'M', 'P'],
                    vec!['K', 'N', 'Q'],
                    vec!['L', 'O', 'R'],
                ],
                across_surfaces: vec![
                    Some("D1".to_string()),
                    Some("D2".to_string()),
                    Some("D3".to_string()),
                ],
                down_surfaces: vec![
                    Some("A1".to_string()),
                    Some("A2".to_string()),
                    Some("A3".to_string()),
                ],
            },
            QuinianCrossword {
                grid1: vec![
                    vec!['A', 'B', 'C'],
                    vec!['D', 'E', 'F'],
                    vec!['G', 'H', 'I'],
                ],
                grid2: vec![
                    vec!['J', 'K', 'L'],
                    vec!['M', 'N', 'O'],
                    vec!['P', 'Q', 'R'],
                ],
                across_surfaces: vec![
                    Some("A1".to_string()),
                    Some("A2".to_string()),
                    Some("A3".to_string()),
                ],
                down_surfaces: vec![
                    Some("D1".to_string()),
                    Some("D2".to_string()),
                    Some("D3".to_string()),
                ],
            },
        ];
        let mut expected_quines = HashSet::new();
        expected_quines.insert(expected_quines_list[0].clone());
        expected_quines.insert(expected_quines_list[1].clone());
        assert_eq!(quines.len(), expected_quines.len());
        assert_eq!(expected_quines, quines);
    }
}
