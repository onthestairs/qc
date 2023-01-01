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
use crate::store::broda_scores::get_broda_scores;
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

    /// min broda score
    #[clap(long)]
    min_broda_score: Option<u32>,
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
    broda_scores: &HashMap<Word, u32>,
    min_broda_score: Option<u32>,
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
    if let Some(min_score) = min_broda_score {
        if *broda_scores.get(solution).unwrap_or(&0) < min_score {
            return false;
        }
    }
    return true;
}

fn filter_clues(
    clues: Vec<(String, Word)>,
    word_frequencies: HashMap<Word, u32>,
    min_word_frequency: Option<u32>,
    broda_scores: &HashMap<Word, u32>,
    min_broda_score: Option<u32>,
) -> Vec<(String, Word)> {
    return clues
        .into_iter()
        .filter(|(surface, solution)| {
            return should_include(
                surface,
                solution,
                &word_frequencies,
                min_word_frequency,
                broda_scores,
                min_broda_score,
            );
        })
        .collect();
}

/// Run the find command
pub fn run(args: Args) {
    let connection = get_connection();
    ensure_results_table_exists(&connection);
    let clues = get_clues().unwrap();
    let word_frequencies = get_words_wiki_frequencies();
    let broda_scores = get_broda_scores();
    let save_crossword = |crossword: &QuinianCrossword,
                          crossword_type: String,
                          words: Vec<Word>,
                          missing_surfaces: usize| {
        let (average_broda_score, min_broda_score) = calculate_broda_scores(&broda_scores, words);
        insert_result_into_table(
            &connection,
            crossword,
            crossword_type,
            missing_surfaces,
            average_broda_score,
            min_broda_score,
        );
    };
    find_qcs(
        clues,
        word_frequencies,
        &broda_scores,
        args.searcher,
        args.min_word_frequency,
        args.min_broda_score,
        args.start_index,
        args.allowed_missing_surfaces,
        &save_crossword,
    );
}

fn calculate_broda_scores(
    broda_scores: &HashMap<Vec<char>, u32>,
    words: Vec<Vec<char>>,
) -> (f64, f64) {
    let scores = words.iter().map(|word| {
        let s = broda_scores.get(word).unwrap_or(&0);
        return *s as f64;
    });

    let total: f64 = scores.clone().sum();
    let min: f64 = scores
        .min_by(|a, b| f64::partial_cmp(a, b).unwrap())
        .unwrap();
    let average = total / words.len() as f64;
    return (average, min);
}

fn find_qcs<F>(
    clues: Vec<(Surface, Word)>,
    word_frequencies: HashMap<Word, u32>,
    broda_scores: &HashMap<Word, u32>,
    searcher: CrosswordType,
    min_word_frequency: Option<u32>,
    min_broda_score: Option<u32>,
    start_index: usize,
    allowed_missing_surfaces: usize,
    on_found: &F,
) where
    F: Fn(&QuinianCrossword, String, Vec<Word>, usize) -> (),
{
    let filtered_clues = filter_clues(
        clues,
        word_frequencies,
        min_word_frequency,
        broda_scores,
        min_broda_score,
    );
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
    use std::ops::Deref;
    use std::sync::Mutex;

    use super::*;

    fn run_test(
        crossword_type: CrosswordType,
        clues: Vec<(&str, &str)>,
        expected_quines_list: Vec<QuinianCrossword>,
        allowed_missing_surfaces: usize,
    ) {
        let clues_munged = clues
            .into_iter()
            .map(|(surface, answer)| return (surface.to_string(), answer.chars().collect()))
            .collect();

        let quines_mutex = Mutex::new(HashSet::new());

        let store_quine = |crossword: &QuinianCrossword,
                           _crossword_type: String,
                           _words: Vec<Word>,
                           _missing_surfaces: usize| {
            let mut quines = quines_mutex.lock().unwrap();
            quines.insert(crossword.clone());
        };
        find_qcs(
            clues_munged,
            HashMap::new(),
            &HashMap::new(),
            crossword_type,
            None,
            None,
            0,
            allowed_missing_surfaces,
            &store_quine,
        );

        let mut expected_quines = HashSet::new();
        for quine in expected_quines_list {
            expected_quines.insert(quine.clone());
        }

        let quines = quines_mutex.lock().unwrap();
        for quine in &*quines {
            crate::generate::qc::print_qc(&quine);
        }
        assert_eq!(quines.len(), expected_quines.len());
        assert_eq!(expected_quines, *quines.deref());
    }

    #[test]
    fn dense3_simple() {
        // Construct the clues so that it should be able to
        // make this crossword (and its transpose)
        // ABC   JKL
        // DEF   MNO
        // GHI   PQR
        // 1. A1     1. D1
        // 2. A2     2. D2
        // 3. A3     3. D3
        let clues = vec![
            ("A1", "ABC"),
            ("A1", "JKL"),
            ("A2", "DEF"),
            ("A2", "MNO"),
            ("A3", "GHI"),
            ("A3", "PQR"),
            // down clues
            ("D1", "ADG"),
            ("D1", "JMP"),
            ("D2", "BEH"),
            ("D2", "KNQ"),
            ("D3", "CFI"),
            ("D3", "LOR"),
        ];
        let expected_quines = vec![
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
        ];

        run_test(CrosswordType::Dense3, clues, expected_quines, 0);
    }

    #[test]
    fn dense3_one_missing() {
        // Construct the clues so that it should be able to
        // make this crossword
        // ABC   JKL
        // DEF   MNO
        // GHI   PQR
        // 1. A1          1. D1
        // 2. A2          2. D2
        // 3. [missing]   3. D3
        let clues = vec![
            ("A1", "ABC"),
            ("A1", "JKL"),
            ("A2", "DEF"),
            ("A2", "MNO"),
            // ("A3", "GHI"),
            // ("A3", "PQR"),
            // down clues
            ("D1", "ADG"),
            ("D1", "JMP"),
            ("D2", "BEH"),
            ("D2", "KNQ"),
            ("D3", "CFI"),
            ("D3", "LOR"),
            // add in "GHI", and "PQR", so that we know
            // the are real words
            ("F1", "GHI"),
            ("F2", "PQR"),
        ];
        let expected_quines = vec![QuinianCrossword {
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
            across_surfaces: vec![Some("A1".to_string()), Some("A2".to_string()), None],
            down_surfaces: vec![
                Some("D1".to_string()),
                Some("D2".to_string()),
                Some("D3".to_string()),
            ],
        }];

        // if we dont allow one missing, then we wont find anything
        run_test(CrosswordType::Dense3, clues.clone(), vec![], 0);
        // but if we do allow one missing, we should only find the one
        // possible qc
        run_test(CrosswordType::Dense3, clues.clone(), expected_quines, 1);
    }

    #[test]
    fn dense3_two_grids_first_row() {
        // Construct the clues so that it should be able to
        // make this crossword
        // ABC   JKL
        // DEF   MNO
        // GHI   PQR
        // 1. A1          1. D1
        // 2. A2          2. D2
        // 3. A3   3. D3
        let clues = vec![
            ("A1", "ABC"),
            ("A1", "JKL"),
            ("A1_2", "ABC"),
            ("A1_2", "JKL"),
            ("A2", "DEF"),
            ("A2", "MNO"),
            ("A3", "GHI"),
            ("A3", "PQR"),
            // down clues
            ("D1", "ADG"),
            ("D1", "JMP"),
            ("D2", "BEH"),
            ("D2", "KNQ"),
            ("D3", "CFI"),
            ("D3", "LOR"),
        ];
        let expected_quines = vec![
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
                    Some("A1_2".to_string()),
                    Some("A2".to_string()),
                    Some("A3".to_string()),
                ],
                down_surfaces: vec![
                    Some("D1".to_string()),
                    Some("D2".to_string()),
                    Some("D3".to_string()),
                ],
            },
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
                    Some("A1_2".to_string()),
                    Some("A2".to_string()),
                    Some("A3".to_string()),
                ],
            },
        ];

        run_test(CrosswordType::Dense3, clues.clone(), expected_quines, 0);
    }

    #[test]
    fn dense3_two_grids_last_row() {
        // Construct the clues so that it should be able to
        // make this crossword
        // ABC   JKL
        // DEF   MNO
        // GHI   PQR
        // 1. A1          1. D1
        // 2. A2          2. D2
        // 3. A3   3. D3
        let clues = vec![
            ("A1", "ABC"),
            ("A1", "JKL"),
            ("A2", "DEF"),
            ("A2", "MNO"),
            ("A3", "GHI"),
            ("A3", "PQR"),
            ("A3_2", "GHI"),
            ("A3_2", "PQR"),
            // down clues
            ("D1", "ADG"),
            ("D1", "JMP"),
            ("D2", "BEH"),
            ("D2", "KNQ"),
            ("D3", "CFI"),
            ("D3", "LOR"),
        ];
        let expected_quines = vec![
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
            // we dont have the following crossword since we assume
            // that there is only one good multi-surface per pair
            // TODO - should we change this?
            // QuinianCrossword {
            //     grid1: vec![
            //         vec!['A', 'B', 'C'],
            //         vec!['D', 'E', 'F'],
            //         vec!['G', 'H', 'I'],
            //     ],
            //     grid2: vec![
            //         vec!['J', 'K', 'L'],
            //         vec!['M', 'N', 'O'],
            //         vec!['P', 'Q', 'R'],
            //     ],
            //     across_surfaces: vec![
            //         Some("A1".to_string()),
            //         Some("A2".to_string()),
            //         Some("A3_2".to_string()),
            //     ],
            //     down_surfaces: vec![
            //         Some("D1".to_string()),
            //         Some("D2".to_string()),
            //         Some("D3".to_string()),
            //     ],
            // },
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
                    Some("A3_2".to_string()),
                ],
            },
        ];

        run_test(CrosswordType::Dense3, clues.clone(), expected_quines, 0);
    }
}
