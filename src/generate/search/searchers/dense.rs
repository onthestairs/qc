//! A searcher for a dense grid

use std::collections::HashMap;
use std::collections::HashSet;

use itertools::Combinations;
use itertools::Itertools;

use super::Pair;
use super::PairStatus;
use super::Searcher;
use crate::generate::data::get_multi_surfaces;
use crate::generate::data::make_ms_pairs;
use crate::generate::data::make_pair_prefix_lookup;
use crate::generate::data::make_pairs_to_surface;
use crate::generate::data::make_word_list_all;
use crate::generate::data::PairPrefixLookup;
use crate::generate::data::Word;
use crate::generate::grid::find_col_prefix;
use crate::generate::grid::get_all_words;
use crate::generate::grid::get_words_in_row_after;
use crate::generate::grid::init_grid;
use crate::generate::grid::make_empty_grid;
use crate::generate::grid::place_word_in_col_mut;
use crate::generate::grid::reset_grid;
use crate::generate::grid::Grid;
use crate::generate::qc::QuinianCrossword;

/// A searcher for a dense crossword
#[derive(Clone)]
pub struct Dense {
    /// The size of the grid
    size: usize,
    /// The clue-pairs
    pairs: Vec<Pair>,
    /// The prefix lookups
    prefix_lookup: PairPrefixLookup,
    /// A map from two words to a surface
    pairs_to_surface: HashMap<(Word, Word), String>,
    /// All the available words
    word_list: HashSet<Word>,
    /// An empty vec (used as a hack)
    empty_vec: Vec<Pair>,
}

/// The stages of placing clues in a dense grid
pub enum DenseStage {
    /// Placing the down clues
    Downs,
}

impl Searcher for Dense {
    type Params = usize;
    type Grids = (Grid, Grid);
    type Surfaces = (Vec<Option<String>>, Vec<Option<String>>);
    type Stage = DenseStage;
    fn new(params: Self::Params, clues: Vec<(String, Word)>) -> Dense {
        let size = params;
        let filtered_clues = clues
            .into_iter()
            .filter(|clue| clue.1.len() == params)
            .collect();
        let multi_surfaces = get_multi_surfaces(&filtered_clues);
        println!("Found {} multi-surfaces", multi_surfaces.len());
        let pairs = make_ms_pairs(&multi_surfaces);
        let pairs_to_surface = make_pairs_to_surface(&pairs);
        let word_list = make_word_list_all(size, &filtered_clues);
        let ms_pairs_cloned = pairs.clone();
        let prefix_lookup = make_pair_prefix_lookup(&ms_pairs_cloned);
        let empty_vec = vec![];
        return Dense {
            size,
            pairs,
            prefix_lookup,
            pairs_to_surface,
            word_list,
            empty_vec,
        };
    }

    fn crossword_type(&self) -> String {
        return format!("dense{}", self.size);
    }

    fn init_grids(&self) -> (Self::Grids, Self::Surfaces) {
        let grid1 = make_empty_grid(self.size);
        let grid2 = make_empty_grid(self.size);
        let across_surfaces = vec![None; self.size];
        let down_surfaces = vec![None; self.size];
        return ((grid1, grid2), (across_surfaces, down_surfaces));
    }

    fn get_initial_stage(&self) -> Self::Stage {
        return DenseStage::Downs;
    }

    fn get_next_stage(&self, _: &Self::Stage) -> Option<Self::Stage> {
        return None;
    }

    fn calculate_number_of_initial_pairs(&self) -> usize {
        let number_of_pairs = self.pairs.len();
        return number_of_pairs * (number_of_pairs - 1) / 2;
    }

    fn get_initial_pairs(
        &self,
    ) -> Combinations<std::slice::Iter<(std::string::String, Vec<char>, Vec<char>)>> {
        return self.pairs.iter().combinations(2);
    }

    fn reset_and_place_initial_pairs(
        &self,
        grids: &mut Self::Grids,
        (across_surfaces, _down_surfaces): &mut Self::Surfaces,
        pairs: &Vec<&Pair>,
    ) {
        reset_grid(&mut grids.0);
        reset_grid(&mut grids.1);
        let (across_surface_1, w11, w12) = pairs[0];
        across_surfaces[0] = Some(across_surface_1.clone());
        let (across_surface_2, w21, w22) = pairs[1];
        across_surfaces[1] = Some(across_surface_2.clone());
        init_grid(&mut grids.0, w11, w21);
        init_grid(&mut grids.1, w12, w22);
    }

    fn get_next_pairs(&self, _: &Self::Stage, grids: &Self::Grids) -> Vec<Vec<&Pair>> {
        return find_possible_downs(&self.prefix_lookup, &self.empty_vec, &grids.0, &grids.1);
    }

    fn place_next_pairs(
        &self,
        _: &Self::Stage,
        grids: &mut Self::Grids,
        surfaces: &mut Self::Surfaces,
        pairs: &Vec<&Pair>,
    ) {
        place_down_clues(&mut grids.0, &mut grids.1, &mut surfaces.1, &pairs);
    }

    fn get_final_statuses(
        &self,
        grids: &Self::Grids,
        (across_surfaces, _down_surfaces): &mut Self::Surfaces,
    ) -> Vec<PairStatus> {
        // check if the final across words are proper words
        let final_words_1 = get_words_in_row_after(&grids.0, 1);
        let final_words_2 = get_words_in_row_after(&grids.1, 1);
        let final_across_pairs = final_words_1.into_iter().zip(final_words_2.into_iter());

        let final_word_statuses: Vec<PairStatus> = final_across_pairs
            .enumerate()
            .map(|(row, (w1, w2))| {
                if self.word_list.contains(w1) && self.word_list.contains(w2) {
                    match self.pairs_to_surface.get(&(w1.clone(), w2.clone())) {
                        Some(surface) => {
                            across_surfaces[2 + row] = Some(surface.clone());
                            PairStatus::HasSurface(surface.clone())
                        }
                        None => PairStatus::Words,
                    }
                } else {
                    PairStatus::NotWords
                }
            })
            .collect();
        return final_word_statuses;
    }

    fn is_happy(&self, grids: &Self::Grids) -> bool {
        return no_duplicates_in_grid(self.size, &grids.0, &grids.1);
    }

    fn get_crossword(
        &self,
        grids: &Self::Grids,
        (across_surfaces, down_surfaces): &Self::Surfaces,
    ) -> QuinianCrossword {
        return QuinianCrossword {
            grid1: grids.0.clone(),
            grid2: grids.1.clone(),
            across_surfaces: across_surfaces.clone(),
            down_surfaces: down_surfaces.clone(),
        };
    }
}

/// Find the possible downs in a dense grid
pub fn find_possible_downs<'a>(
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

/// Place down clues in a dense grid
pub fn place_down_clues(
    g1: &mut Grid,
    g2: &mut Grid,
    surfaces: &mut Vec<Option<String>>,
    down_combos: &Vec<&(String, Word, Word)>,
) {
    let mut col = 0;
    for (surface, w1, w2) in down_combos {
        place_word_in_col_mut(g1, col, w1);
        place_word_in_col_mut(g2, col, w2);
        surfaces[col] = Some(surface.clone());
        col += 1;
    }
}

/// Check for duplicates in two dense grids
pub fn no_duplicates_in_grid(size: usize, g1: &Grid, g2: &Grid) -> bool {
    let words1 = get_all_words(g1);
    let words2 = get_all_words(g2);
    let mut all_words: HashSet<Word> = words1.into_iter().collect();
    for word in words2 {
        all_words.insert(word);
    }
    let expected_len = (2 * size) * 2;
    return expected_len == all_words.len();
}
