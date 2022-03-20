//! Generate all words

pub mod searchers;

use itertools::Itertools;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::time::Instant;

use searchers::dense::{find_possible_downs, no_duplicates_in_grid, place_down_clues};

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

use super::qc::QuinianCrossword;
use searchers::PairStatus;
use searchers::Searcher;

/// Make a hash of a crossword
pub fn hash_crossword(crossword: &QuinianCrossword) -> u64 {
    let mut hasher = DefaultHasher::new();
    crossword.hash(&mut hasher);
    return hasher.finish();
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

            // check if the final across words are proper words
            let final_words_1 = get_words_in_row_after(&g1, 1);
            let final_words_2 = get_words_in_row_after(&g2, 1);
            let final_across_pairs = final_words_1.into_iter().zip(final_words_2.into_iter());

            let final_word_statuses: Vec<PairStatus> = final_across_pairs
                .map(|(w1, w2)| {
                    if word_list.contains(w1) && word_list.contains(w2) {
                        match pairs_to_surface.get(&(w1.clone(), w2.clone())) {
                            Some(surface) => PairStatus::HasSurface(surface.clone()),
                            None => PairStatus::Words,
                        }
                    } else {
                        PairStatus::NotWords
                    }
                })
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

            if illegal_count == 0
                && no_surface_count <= allowed_missing_surfaces
                && no_duplicates_in_grid(size, &g1, &g2)
            {
                let mut across_surfaces = vec![
                    Some(across_surface_1.clone()),
                    Some(across_surface_2.clone()),
                ];
                for final_word_status in final_word_statuses {
                    match final_word_status {
                        PairStatus::HasSurface(surface) => across_surfaces.push(Some(surface)),
                        PairStatus::Words => across_surfaces.push(None),
                        PairStatus::NotWords => todo!(),
                    }
                }
                let down_surfaces = down_combo.iter().map(|(s, _, _)| Some(s.clone())).collect();
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
            let percent = (i as f32 / number_of_combos as f32) * 100.0;
            println!("Done {i} ({percent}%) in {duration:?}");
            batch_start_time = Instant::now();
        }
    }
}

/// Find grids using the given searcher
pub fn find_grids_with_searcher<T, F>(
    start_index: usize,
    allowed_missing_surfaces: usize,
    searcher: &T,
    on_found: F,
) where
    T: Searcher,
    F: Fn(&QuinianCrossword, String, usize) -> (),
{
    let mut i = 0;
    let crossword_type = searcher.crossword_type();
    let mut batch_start_time = Instant::now();
    // get all the initial pairs
    let mut grids = searcher.init_grids();
    let initial_pairs = searcher.get_initial_pairs();
    for pairs in initial_pairs {
        i += 1;
        if i < start_index {
            continue;
        }
        searcher.reset_and_place_initial_pairs(&mut grids, &pairs);
        for other_pairs in searcher.get_other_pairs(&grids) {
            searcher.place_other_pairs(&mut grids, &other_pairs);
            let final_word_statuses = searcher.get_final_statuses(&grids);

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

            if illegal_count == 0
                && no_surface_count <= allowed_missing_surfaces
                && searcher.is_happy(&grids)
            {
                let quinian_crossword = searcher.get_crossword(&grids);
                on_found(&quinian_crossword, crossword_type.clone(), score);
            }
            if i % 10000 == 0 {
                let duration = batch_start_time.elapsed();
                // let percent = (i as f32 / number_of_combos as f32) * 100.0;
                let percent = (i as f32 / 100 as f32) * 100.0;
                println!("Done {i} ({percent}%) in {duration:?}");
                batch_start_time = Instant::now();
            }
        }
    }
}

/// Find all quinian grids for the given clues
pub fn find_solutions<F>(
    size: usize,
    allowed_missing_surfaces: usize,
    start_index: usize,
    clues: Vec<(String, Word)>,
    on_found: F,
) where
    F: Fn(&QuinianCrossword, usize, usize) -> (),
{
    // prepare the pre_computed data structures
    let multi_surfaces = get_multi_surfaces(&clues);
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
