//! Generate all words

use itertools::Itertools;
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::time::Instant;

use crate::generate::data::make_first_and_third_prefix_lookup;
use crate::generate::grid::{make_sparse_grid, init_sparse_grid, sparse_place_word_in_row_mut};

use super::data::{get_multi_surfaces, MultiSurface, PairFirstAndThirdLookup};
use super::data::make_ms_pairs;
use super::data::make_pair_prefix_lookup;
use super::data::make_pairs_to_surface;
use super::data::make_word_list_all;
use super::grid::{get_words_in_row_after, find_col_mask, find_row_mask, sparse_get_all_words};
use super::grid::init_grid;
use super::grid::make_empty_grid;
use super::grid::reset_grid;
use super::grid::reset_sparse_grid;

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
    e: &'a Vec<MultiSurface>,
    grid1: &Grid,
    grid2: &Grid,
) -> Vec<Vec<&'a MultiSurface>> {
    let size = grid1.len();
    // find the possible pairs in each column
    let ds: Vec<&Vec<MultiSurface>> = (0..size)
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

fn sparse_find_possible_downs<'a>(
    lookup: &'a PairFirstAndThirdLookup,
    // weird hack so that i can use the default in the
    // map lookup
    e: &'a Vec<MultiSurface>,
    grid1: &Grid,
    grid2: &Grid,
) -> Vec<Vec<&'a MultiSurface>> {
    // let size = grid1.len();
    let down_cols = vec![0, 2, 4];
    // find the possible pairs in each column
    let ds: Vec<&Vec<MultiSurface>> = down_cols.into_iter()
        .map(|col| {
            // let prefix1 = find_col_prefix(grid1, col, 2);
            // let prefix2 = find_col_prefix(grid2, col, 2);
            let prefix1 = find_col_mask(grid1, col, vec![0,2]);
            let prefix2 = find_col_mask(grid2, col, vec![0,2]);
            let maybe_down_pairs = lookup.get(&(prefix1, prefix2));
            let down_pairs = maybe_down_pairs.unwrap_or(e);
            return down_pairs;
        })
        .collect();
    // Get every combo of possible placements in the columns
    return ds.into_iter().multi_cartesian_product().collect();
}

// place a single set of down clues into the grid
fn sparse_place_down_clues(g1: &mut Grid, g2: &mut Grid, down_combos: &Vec<&MultiSurface>) {
    let mut col = 0;
    for (_, w1, w2) in down_combos {
        place_word_in_col_mut(g1, col, w1);
        place_word_in_col_mut(g2, col, w2);
        col += 2;
    }
}

fn place_down_clues(g1: &mut Grid, g2: &mut Grid, down_combos: &Vec<&MultiSurface>) {
    let mut col = 0;
    for (_, w1, w2) in down_combos {
        place_word_in_col_mut(g1, col, w1);
        place_word_in_col_mut(g2, col, w2);
        col += 1;
    }
}

fn sparse_no_duplicates_in_grid(g1: &Grid, g2: &Grid) -> bool {
    let words1 = sparse_get_all_words(g1);
    let words2 = sparse_get_all_words(g2);
    let mut all_words: HashSet<Word> = words1.into_iter().collect();
    for word in words2 {
        all_words.insert(word);
    }
    let expected_len = 12;
    return expected_len == all_words.len();
}

fn no_duplicates_in_grid(size: usize, g1: &Grid, g2: &Grid) -> bool {
    let words1 = get_all_words(g1);
    let words2 = get_all_words(g2);
    let mut all_words: HashSet<Word> = words1.into_iter().collect();
    for word in words2 {
        all_words.insert(word);
    }
    let expected_len = (2 * size) * 2;
    return expected_len == all_words.len();
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
    pairs: Vec<MultiSurface>,
    // prefix_lookup: PairPrefixLookup,
    mask_lookup: PairFirstAndThirdLookup,
    pairs_to_surface: HashMap<(Word, Word), String>,
    word_list: HashSet<Word>,
    on_found: F,
) where
    F: Fn(&QuinianCrossword, usize, usize) -> (),
{
    assert!(size == 5);
    // was changing this code so that it can build my sparse grid. Actually might 
    // have to change quite a lot of the dependent code like find_possible_downs and 
    // place_down_clues. Could I 
    // just make (some of) them generic?

    // let mut g1 = make_empty_grid(size);
    // let mut g2 = make_empty_grid(size);
    let mut g1 = make_sparse_grid();
    let mut g2 = make_sparse_grid();
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
        reset_sparse_grid(&mut g1);
        reset_sparse_grid(&mut g2);
        // place the words in the top 2 rows
        init_sparse_grid(&mut g1, w11, w21);
        init_sparse_grid(&mut g2, w12, w22);
        // find all the ways we can place a pair in all of the
        // columns
        // empty_vec is a weird hack. see definition of func
        let empty_vec = vec![];
        let down_combos = sparse_find_possible_downs(&mask_lookup, 
            &empty_vec, &g1, &g2);
        for down_combo in down_combos {
            sparse_place_down_clues(&mut g1, &mut g2, &down_combo);



            // // check if the final across words are proper words
            // let final_words_1 = get_words_in_row_after(&g1, 3);
            // let final_words_2 = get_words_in_row_after(&g2, 3);
            // let final_across_pairs = final_words_1.into_iter().zip(final_words_2.into_iter());

            // let final_word_statuses: Vec<PairStatus> = final_across_pairs
            //     .map(|(w1, w2)| {
            //         if word_list.contains(w1) && word_list.contains(w2) {
            //             match pairs_to_surface.get(&(w1.clone(), w2.clone())) {
            //                 Some(surface) => PairStatus::HasSurface(surface.clone()),
            //                 None => PairStatus::Words,
            //             }
            //         } else {
            //             PairStatus::NotWords
            //         }
            //     })
            //     .collect();
            // let mut illegal_count = 0;
            // let mut no_surface_count = 0;
            // for final_word_status in &final_word_statuses {
            //     match final_word_status {
            //         PairStatus::HasSurface(_) => {}
            //         PairStatus::Words => no_surface_count += 1,
            //         PairStatus::NotWords => illegal_count += 1,
            //     }
            // }
            let empty_vec = vec![];
            let final_row_candidates = 
                find_final_row_candidates(&mask_lookup, &g1, &g2, &empty_vec);
            for (clue, w1, w2) in final_row_candidates {
                sparse_place_word_in_row_mut(&mut g1, 4, w1);
                sparse_place_word_in_row_mut(&mut g2, 4, w2);

                if sparse_no_duplicates_in_grid(&g1, &g2) {


                let across_surfaces = vec![across_surface_1.clone(), 
                across_surface_2.clone(), clue.clone()];
                let down_surfaces = down_combo.iter().map(|(s, _, _)| s).cloned().collect();
                let solution = QuinianCrossword {
                    grid1: g1.clone(),
                    grid2: g2.clone(),
                    across_surfaces,
                    down_surfaces,
                };
                on_found(&solution, size, 6);
                }

            }


            // let score = no_surface_count;

            // if illegal_count == 0
            //     && no_surface_count <= allowed_missing_surfaces
            //     && no_duplicates_in_grid(size, &g1, &g2)
            // {
            //     let mut across_surfaces = vec![across_surface_1.clone(), across_surface_2.clone()];
            //     for final_word_status in final_word_statuses {
            //         match final_word_status {
            //             PairStatus::HasSurface(surface) => across_surfaces.push(surface),
            //             PairStatus::Words => across_surfaces.push("[To write...]".to_string()),
            //             PairStatus::NotWords => todo!(),
            //         }
            //     }
            //     let down_surfaces = down_combo.iter().map(|(s, _, _)| s).cloned().collect();
            //     let solution = QuinianCrossword {
            //         grid1: g1.clone(),
            //         grid2: g2.clone(),
            //         across_surfaces,
            //         down_surfaces,
            //     };
            //     on_found(&solution, size, score);
            // }
        }
        if i % 10000 == 0 {
            let duration = batch_start_time.elapsed();
            let percent = (i as f32 / number_of_combos as f32) * 100.0;
            println!("Done {i} ({percent}%) in {duration:?}");
            batch_start_time = Instant::now();
        }
    }
}

fn find_final_row_candidates<'a>(
    lookup: &'a PairFirstAndThirdLookup,
    grid1: &Grid,
    grid2: &Grid,
    // weird hack so that i can use the default in the
    // map lookup
    empty: &'a Vec<MultiSurface>,
) -> Vec<&'a MultiSurface> {
    // lookup those that fit the first two letters...
    let mask1 = find_row_mask(grid1, 4, vec![0,2]);
    let mask2 = find_row_mask(grid2, 4, vec![0,2]);
    let maybe_candidates = lookup.get(&(mask1, mask2));
    if maybe_candidates.is_none() {
        return vec![];
    }
    //inlined this because rust madness
    let matches_last = |candidate: &MultiSurface| {
        let (_clue, w1, w2) = candidate;
        //sanity check
        assert!(w1[0] == grid1[4][0]);
        assert!(w1[2] == grid1[4][2]);
        assert!(w2[0] == grid2[4][0]);
        assert!(w2[2] == grid2[4][2]);
        return (w1[4] == grid1[4][4]) && (w2[4] == grid2[4][4]);
    };
    return maybe_candidates.unwrap_or(empty)
     .into_iter()
     .filter(|candidate| { 
         return matches_last(*candidate);
        })
    //  .filter(
    //  |candidate: MultiSurface| {
    //     let (clue, w1, w2) = candidate;
    //     //sanity check
    //     assert!(w1[0] == grid1[4][0]);
    //     assert!(w1[2] == grid1[4][2]);
    //     assert!(w2[0] == grid2[4][0]);
    //     assert!(w2[2] == grid2[4][2]);
    //     return (w1[4] == grid1[4][4]) && (w2[4] == grid2[4][4]);
    // })
    // matches_last)
     .collect();

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
    // let pair_prefix_lookup = make_pair_prefix_lookup(&ms_pairs_cloned);
    let pair_prefix_lookup = make_first_and_third_prefix_lookup(&ms_pairs_cloned);
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
