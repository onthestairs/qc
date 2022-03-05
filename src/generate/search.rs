//! Generate all words

use itertools::Itertools;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::time::Instant;

use super::data::get_multi_surfaces;
use super::data::make_ms_pairs;
use super::data::make_pair_prefix_lookup;
use super::data::make_pairs_to_surface;
use super::data::make_word_list_all;
use super::grid::get_words_in_row_after;
use super::grid::init_grid;
use super::grid::make_empty_grid;
use super::grid::reset_grid;

use super::data::PairPrefixLookup;
use super::data::Word;
use super::grid::find_col_prefix;
use super::grid::get_all_words;
use super::grid::place_word_in_col_mut;
use super::grid::Grid;
use super::qc::QuinianCrossword;

fn find_possible_downs<'a>(
    lookup: &'a PairPrefixLookup,
    // weird hack so that i can use the default in the
    // map lookup
    e: &'a Vec<(String, Word, Word)>,
    grid1: &Grid,
    grid2: &Grid,
) -> Vec<Vec<&'a (String, Word, Word)>> {
    let size = grid1.len();
    // find the possible pairs in each column
    let ds: Vec<&Vec<(String, Word, Word)>> = (0..size)
        .map(|col| {
            let prefix1 = find_col_prefix(grid1, col, 2);
            let prefix2 = find_col_prefix(grid2, col, 2);
            let maybe_down_pairs = lookup.get(&(prefix1, prefix2));
            let down_pairs = maybe_down_pairs.unwrap_or(e);
            return down_pairs;
        })
        .collect();
    // Get every combo of possible placements in the columns
    return ds.into_iter().multi_cartesian_product().collect();
}

fn place_down_clues(g1: &mut Grid, g2: &mut Grid, down_combos: &Vec<&(String, Word, Word)>) {
    let mut col = 0;
    for (_, w1, w2) in down_combos {
        place_word_in_col_mut(g1, col, w1);
        place_word_in_col_mut(g2, col, w2);
        col += 1;
    }
}

fn no_duplicates_in_grid(g1: &Grid, g2: &Grid) -> bool {
    let mut words = get_all_words(g1);
    let ws2 = get_all_words(g2);
    words.extend(ws2);
    let words_len = words.len();

    let words_set: HashSet<Word> = words.into_iter().collect();
    return words_len == words_set.len();
}

/// Make a hash of a crossword
pub fn hash_crossword(crossword: &QuinianCrossword) -> u64 {
    let mut hasher = DefaultHasher::new();
    crossword.hash(&mut hasher);
    return hasher.finish();
}

enum PairStatus {
    HasSurface(String),
    Words,
    NotWords,
}

fn find_grids<F>(
    start_index: usize,
    size: usize,
    allowed_missing_surfaces: usize,
    pairs: Vec<(String, Word, Word)>,
    prefix_lookup: PairPrefixLookup,
    pairs_to_surface: HashMap<(Word, Word), String>,
    word_list: HashSet<Word>,
    on_found: F,
) where
    F: Fn(&QuinianCrossword, usize, usize) -> (),
{
    let mut g1 = make_empty_grid(size);
    let mut g2 = make_empty_grid(size);
    let number_of_combos = (pairs.len() * (pairs.len() - 1)) / 2;
    println!("Found {} combos", number_of_combos);
    let mut i = 0;
    let mut batch_start_time = Instant::now();
    // get every combination of 2 pairs, we
    // will place them in the top two rows of each grid
    for pair in pairs.iter().combinations(2) {
        i += 1;
        if i < start_index {
            continue;
        }
        let (across_surface_1, w11, w12) = pair[0];
        let (across_surface_2, w21, w22) = pair[1];
        // reset both the grids
        reset_grid(&mut g1);
        reset_grid(&mut g2);
        // place the words in the top 2 rows
        init_grid(&mut g1, w11, w21);
        init_grid(&mut g2, w12, w22);
        // find all the ways we can place a pair in all of the
        // columns
        // empty_vec is a weird hack. see definition of func
        let empty_vec = vec![];
        let down_combos = find_possible_downs(&prefix_lookup, &empty_vec, &g1, &g2);
        for down_combo in down_combos {
            place_down_clues(&mut g1, &mut g2, &down_combo);

            // ignore any grids with duplicate words
            if !no_duplicates_in_grid(&g1, &g2) {
                continue;
            }

            // check if the final across words are proper words
            let final_words_1 = get_words_in_row_after(&g1, 1);
            let final_words_2 = get_words_in_row_after(&g2, 1);
            let final_across_pairs = final_words_1.into_iter().zip(final_words_2.into_iter());

            let final_word_statuses: Vec<PairStatus> = final_across_pairs
                .map(
                    |(w1, w2)| match pairs_to_surface.get(&(w1.clone(), w2.clone())) {
                        Some(surface) => PairStatus::HasSurface(surface.clone()),
                        None => {
                            if word_list.contains(w1) && word_list.contains(w2) {
                                PairStatus::Words
                            } else {
                                PairStatus::NotWords
                            }
                        }
                    },
                )
                .collect();
            let mut illegal_count = 0;
            let mut no_surface_count = 0;
            for final_word_status in &final_word_statuses {
                match final_word_status {
                    PairStatus::HasSurface(_) => {}
                    PairStatus::Words => no_surface_count += 1,
                    PairStatus::NotWords => illegal_count += 1,
                }
            }
            let score = no_surface_count;

            if illegal_count == 0 && no_surface_count <= allowed_missing_surfaces {
                let mut across_surfaces = vec![across_surface_1.clone(), across_surface_2.clone()];
                for final_word_status in final_word_statuses {
                    match final_word_status {
                        PairStatus::HasSurface(surface) => across_surfaces.push(surface),
                        PairStatus::Words => across_surfaces.push("[To write...]".to_string()),
                        PairStatus::NotWords => todo!(),
                    }
                }
                let down_surfaces = down_combo.iter().map(|(s, _, _)| s).cloned().collect();
                let solution = QuinianCrossword {
                    grid1: g1.clone(),
                    grid2: g2.clone(),
                    across_surfaces,
                    down_surfaces,
                };
                on_found(&solution, size, score);
            }
        }
        if i % 10000 == 0 {
            let duration = batch_start_time.elapsed();
            println!("Done {i} in {duration:?}");
            batch_start_time = Instant::now();
        }
    }
}

/// Find all quinian grids for the given clues
pub fn find_solutions<F>(
    size: usize,
    allowed_missing_surfaces: usize,
    start_index: usize,
    clues: Vec<(String, String)>,
    on_found: F,
) where
    F: Fn(&QuinianCrossword, usize, usize) -> (),
{
    // prepare the pre_computed data structures
    let multi_surfaces = get_multi_surfaces(&clues, size);
    println!("Found {} multi-surfaces", multi_surfaces.len());
    let ms_pairs = make_ms_pairs(&multi_surfaces);
    let pairs_to_surface = make_pairs_to_surface(&ms_pairs);
    // let word_list = make_word_list(&ms_pairs);
    let word_list = make_word_list_all(size, &clues);
    println!("Found {} words", word_list.len());
    println!("Found {} pairs", ms_pairs.len());
    let ms_pairs_cloned = ms_pairs.clone();
    let pair_prefix_lookup = make_pair_prefix_lookup(&ms_pairs_cloned);
    println!("made double prefix lookup");
    // find any good grids
    find_grids(
        start_index,
        size,
        allowed_missing_surfaces,
        ms_pairs,
        pair_prefix_lookup,
        pairs_to_surface,
        word_list,
        on_found,
    );
}
