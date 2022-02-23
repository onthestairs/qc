//! Generate all words

use itertools::Itertools;
use std::collections::{HashMap, HashSet};

use super::data::get_multi_surfaces;
use super::data::make_ms_pairs;
use super::data::make_pair_prefix_lookup;
use super::data::make_pairs_to_surface;
use super::data::make_word_list;
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

fn find_possible_downs(
    lookup: &PairPrefixLookup,
    grid1: &Grid,
    grid2: &Grid,
) -> Vec<Vec<(String, Word, Word)>> {
    let size = grid1.len();
    // find the possible pairs in each column
    let ds: Vec<Vec<(String, Word, Word)>> = (0..size)
        .map(|col| {
            let prefix1 = find_col_prefix(grid1, col, 2);
            let prefix2 = find_col_prefix(grid2, col, 2);
            return lookup.get(&(prefix1, prefix2)).unwrap_or(&vec![]).clone();
        })
        .collect();
    // Get every combo of possible placements in the columns
    return ds.into_iter().multi_cartesian_product().clone().collect();
}

fn place_down_clues(g1: &mut Grid, g2: &mut Grid, down_combos: &Vec<(String, Word, Word)>) {
    let mut col = 0;
    for (_, w1, w2) in down_combos {
        place_word_in_col_mut(g1, col, &w1);
        place_word_in_col_mut(g2, col, &w2);
        col += 1;
    }
}

fn no_duplicates_in_grid(g1: &Grid, g2: &Grid) -> bool {
    let mut words = get_all_words(g1);
    let ws2 = get_all_words(g2);
    words.extend(ws2);

    let words_set: HashSet<Word> = words.iter().cloned().collect();
    return words.len() == words_set.len();
}

/// A quinian crossword
#[derive(PartialEq, Eq, Hash)]
pub struct QuinianCrossword {
    grid1: Grid,
    grid2: Grid,
    across_surfaces: Vec<String>,
    down_surfaces: Vec<String>,
}

/// print a solution
pub fn print_solution(solution: &QuinianCrossword) {
    println!("Across");
    for (i, surface) in solution.across_surfaces.iter().enumerate() {
        println!("{i}. {surface}");
    }
    println!("Down");
    for (i, surface) in solution.down_surfaces.iter().enumerate() {
        println!("{i}. {surface}");
    }

    for i in 0..solution.grid1.len() {
        let row1: String = solution.grid1[i].iter().collect();
        let row2: String = solution.grid2[i].iter().collect();
        println!("{row1} | {row2}")
    }
}

fn find_grids(
    size: usize,
    pairs: Vec<(String, Word, Word)>,
    prefix_lookup: PairPrefixLookup,
    pairs_to_surface: HashMap<(Word, Word), String>,
    _word_list: HashSet<Word>,
) -> HashSet<QuinianCrossword> {
    let mut g1 = make_empty_grid(size);
    let mut g2 = make_empty_grid(size);
    let mut solutions: HashSet<QuinianCrossword> = HashSet::new();
    let number_of_combos = (pairs.len() * (pairs.len() - 1)) / 2;
    println!("Found {} combos", number_of_combos);
    let mut i = 0;
    // get every combination of 2 pairs, we
    // will place them in the top two rows of each grid
    for pair in pairs.iter().combinations(2) {
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
        let down_combos = find_possible_downs(&prefix_lookup, &g1, &g2);
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

            // if final_pairs.all(|(w1, w2)| word_list.contains(&w1) && word_list.contains(&w2)) {
            let final_maybe_surfaces: Vec<Option<String>> = final_across_pairs
                .map(|(w1, w2)| pairs_to_surface.get(&(w1, w2)).map(|s| s.clone()))
                .collect();
            // dbg!(&final_maybe_surfaces);
            let final_surfaces: Vec<String> = Itertools::flatten(final_maybe_surfaces.iter())
                .cloned()
                .collect();

            if final_maybe_surfaces.len() == final_surfaces.len() {
                // dbg!(&final_surfaces);
                // let maybe_final_surface = pairs_to_surface.get(&(lw1, lw2));
                // if let Some(final_surface) = maybe_final_surface {
                let mut across_surfaces = vec![across_surface_1.clone(), across_surface_2.clone()];
                for final_surface in final_surfaces {
                    across_surfaces.push(final_surface.clone());
                }
                let down_surfaces = down_combo.iter().map(|(s, _, _)| s).cloned().collect();
                let solution = QuinianCrossword {
                    grid1: g1.clone(),
                    grid2: g2.clone(),
                    across_surfaces,
                    down_surfaces,
                };
                print_solution(&solution);
                println!("====================");
                println!("====================");
                solutions.insert(solution);
            }
        }
        if i % 10000 == 0 {
            println!("Done {i}");
        }
        i += 1;
    }
    return solutions;
}

/// Find all quinian grids for the given clues
pub fn find_solutions(size: usize, clues: Vec<(String, String)>) -> HashSet<QuinianCrossword> {
    // prepare the pre_computed data structures
    let multi_surfaces = get_multi_surfaces(clues, size);
    println!("Found {} multi-surfaces", multi_surfaces.len());
    let ms_pairs = make_ms_pairs(&multi_surfaces);
    let pairs_to_surface = make_pairs_to_surface(&ms_pairs);
    let word_list = make_word_list(&ms_pairs);
    println!("Found {} words", word_list.len());
    println!("Found {} pairs", ms_pairs.len());
    let pair_prefix_lookup = make_pair_prefix_lookup(&ms_pairs);
    println!("made double prefix lookup");
    // find any good grids
    let solutions = find_grids(
        size,
        ms_pairs,
        pair_prefix_lookup,
        pairs_to_surface,
        word_list,
    );
    return solutions;
}
