//! A searcher for a dense grid

use std::collections::HashMap;
use std::collections::HashSet;

use itertools::Itertools;
use itertools::Permutations;

use super::Pair;
use super::PairStatus;
use super::Searcher;
use crate::generate::data::get_multi_surfaces;
use crate::generate::data::make_ms_pairs;
use crate::generate::data::PairPrefixLookup;
use crate::generate::data::Surface;
use crate::generate::data::Word;
use crate::generate::grid::get_word_in_col;
use crate::generate::grid::get_word_in_row;
use crate::generate::grid::make_empty_grid;
use crate::generate::grid::place_word_in_col_mut;
use crate::generate::grid::place_word_in_row_mut;
use crate::generate::grid::Grid;
use crate::generate::qc::QuinianCrossword;

type MultiSurface = (Surface, Word, Word);
type MaskLookup = HashMap<(Word, Word), Vec<MultiSurface>>;

/// A searcher for a dense crossword
#[derive(Clone)]
pub struct Alternating {
    /// the size of the grid
    size: usize,
    /// Number of clues (precomputed)
    number_of_clues: usize,
    /// All the clue pairs
    pairs: Vec<Pair>,
    /// A lookup for the mask
    mask_lookup: MaskLookup,
    /// Hack - we need this lying around to make the lifetimes agree
    empty_vec: Vec<Pair>,
}

/// Make a lookup from the 1st and 3rd characters of each word in a pair, to the
/// possible pairs
pub fn make_mask_lookup(pairs: &Vec<MultiSurface>) -> MaskLookup {
    let mut lookup: PairPrefixLookup = HashMap::new();
    for (s, w1, w2) in pairs {
        let lookup1 = vec![w1[0], w1[2]];
        let lookup2 = vec![w2[0], w2[2]];
        let key_words = lookup.entry((lookup1, lookup2)).or_insert(vec![]);
        (*key_words).push((s.clone(), w1.clone(), w2.clone()));
    }
    return lookup;
}

/// Make an empty sparse grid - this is a 5x5 grid with four clues around the edges
/// and two crossing in the middle:
///  S I M O N
///  I . A . O   
///  M A X I M
///  O . I . I
///  N O M I S
pub fn make_sparse_grid(size: usize) -> Grid {
    let mut grid = make_empty_grid(size);
    reset_sparse_grid(&mut grid);
    return grid;
}

/// Reset all the values in the grid
pub fn reset_sparse_grid(g: &mut Grid) {
    for row in 0..g.len() {
        for col in 0..g[row].len() {
            let char = if row % 2 == 1 && col % 2 == 1 {
                '#'
            } else {
                '.'
            };
            g[row][col] = char;
        }
    }
}

/// Place a word in a row in place
pub fn sparse_place_word_in_row_mut(grid: &mut Grid, row: usize, word: &Word) {
    grid[row] = word.clone()
}

/// Place a word in a col in place
pub fn sparse_place_word_in_col_mut(grid: &mut Grid, col: usize, word: &Word) {
    for row in 0..word.len() {
        grid[row][col] = word[row];
    }
}

/// Initialise two grids with a word for each
pub fn init_sparse_grid(g: &mut Grid, w1: &Word, w2: &Word) {
    sparse_place_word_in_row_mut(g, 0, w1);
    sparse_place_word_in_row_mut(g, 2, w2);
}

// place a single set of down clues into the grid
fn sparse_place_down_clues(
    g1: &mut Grid,
    g2: &mut Grid,
    surfaces: &mut Vec<Option<String>>,
    down_combos: &Vec<&MultiSurface>,
) {
    let mut col = 0;
    for (s, w1, w2) in down_combos {
        surfaces[col / 2] = Some(s.clone());
        place_word_in_col_mut(g1, col, w1);
        place_word_in_col_mut(g2, col, w2);
        col += 2;
    }
}

fn sparse_place_final_across_clues(
    g1: &mut Grid,
    g2: &mut Grid,
    surfaces: &mut Vec<Option<Surface>>,
    across_combos: &Vec<&MultiSurface>,
) {
    let mut row = 4;
    for (s, w1, w2) in across_combos {
        surfaces[row / 2] = Some(s.clone());
        place_word_in_row_mut(g1, row, w1);
        place_word_in_row_mut(g2, row, w2);
        row += 2;
    }
}

/// Get the mask specified in the given column
pub fn find_col_mask(grid: &Grid, col: usize, mask: Vec<usize>) -> Word {
    let mut prefix = vec![];
    for row in mask {
        prefix.push(grid[row][col])
    }
    return prefix;
}

/// Get the mask specified in the given row
pub fn find_row_mask(grid: &Grid, row: usize, mask: Vec<usize>) -> Word {
    let mut prefix = vec![];
    for col in mask {
        prefix.push(grid[row][col])
    }
    return prefix;
}

/// Get all the words in the grid (across and down)
pub fn sparse_get_all_words(size: usize, g: &Grid) -> Vec<Word> {
    let rows = (0..size).step_by(2);
    let mut words: Vec<Word> = rows
        .into_iter()
        .map(|row| {
            return get_word_in_row(g, row).clone();
        })
        .collect();
    let cols = (0..size).step_by(2);
    let mut downs: Vec<Word> = cols
        .into_iter()
        .map(|col| {
            return get_word_in_col(g, col);
        })
        .collect();
    words.append(&mut downs);
    return words;
}

fn sparse_no_duplicates_in_grid(size: usize, g1: &Grid, g2: &Grid) -> bool {
    let words1 = sparse_get_all_words(size, g1);
    let words2 = sparse_get_all_words(size, g2);
    let expected_len = words1.len() + words2.len();
    let mut all_words: HashSet<Word> = words1.into_iter().collect();
    for word in words2 {
        all_words.insert(word);
    }
    return expected_len == all_words.len();
}

fn sparse_find_possible_downs<'a>(
    size: usize,
    lookup: &'a MaskLookup,
    // weird hack so that i can use the default in the
    // map lookup
    e: &'a Vec<MultiSurface>,
    grid1: &Grid,
    grid2: &Grid,
) -> Vec<Vec<&'a MultiSurface>> {
    let down_cols = (0..size).step_by(2);
    // find the possible pairs in each column
    let ds: Vec<&Vec<MultiSurface>> = down_cols
        .into_iter()
        .map(|col| {
            let prefix1 = find_col_mask(grid1, col, vec![0, 2]);
            let prefix2 = find_col_mask(grid2, col, vec![0, 2]);
            let maybe_down_pairs = lookup.get(&(prefix1, prefix2));
            let down_pairs = maybe_down_pairs.unwrap_or(e);
            return down_pairs;
        })
        .collect();
    // Get every combo of possible placements in the columns
    return ds.into_iter().multi_cartesian_product().collect();
}

fn sparse_find_possible_final_acrosses<'a>(
    size: usize,
    lookup: &'a MaskLookup,
    grid1: &Grid,
    grid2: &Grid,
    // weird hack so that i can use the default in the
    // map lookup
    empty: &'a Vec<MultiSurface>,
) -> Vec<Vec<&'a MultiSurface>> {
    let across_rows = (4..size).step_by(2);
    // find the possible pairs in each column
    let ds: Vec<Vec<&MultiSurface>> = across_rows
        .into_iter()
        .map(|row| {
            let prefix1 = find_row_mask(grid1, row, vec![0, 2]);
            let prefix2 = find_row_mask(grid2, row, vec![0, 2]);
            let maybe_across_pairs = lookup.get(&(prefix1, prefix2));
            let across_pairs = maybe_across_pairs.unwrap_or(empty);
            let filtered_across_pairs = across_pairs
                .into_iter()
                .filter(|(_, w1, w2)| {
                    let cols = (4..size).step_by(2);
                    return cols
                        .into_iter()
                        .all(|col| (w1[col] == grid1[row][col]) && (w2[col] == grid2[row][col]));
                })
                .collect();
            return filtered_across_pairs;
        })
        .collect();
    // Get every combo of possible placements in the columns
    return ds.into_iter().multi_cartesian_product().collect();
}

/// The different stages for placing clues in a grid
pub enum AlternatingStage {
    /// Placing the down clues
    Downs,
    /// Placing the final across clues
    FinalAcrosses,
}

impl Searcher for Alternating {
    type Params = usize;
    type Grids = (Grid, Grid);
    type Surfaces = (Vec<Option<String>>, Vec<Option<String>>);
    type Stage = AlternatingStage;
    fn new(params: Self::Params, clues: Vec<(String, Word)>) -> Alternating {
        let size = params;
        let filtered_clues = clues
            .into_iter()
            .filter(|clue| clue.1.len() == params)
            .collect();
        let multi_surfaces = get_multi_surfaces(&filtered_clues);
        println!("Found {} multi-surfaces", multi_surfaces.len());
        let pairs = make_ms_pairs(&multi_surfaces);
        let ms_pairs_cloned = pairs.clone();
        let mask_lookup = make_mask_lookup(&ms_pairs_cloned);
        let empty_vec = vec![];

        // figure out how many clues we have
        let number_of_clues = if size % 2 == 0 {
            size / 2
        } else {
            (size / 2) + 1
        };
        return Alternating {
            size,
            number_of_clues,
            pairs,
            mask_lookup,
            empty_vec,
        };
    }

    fn crossword_type(&self) -> String {
        return format!("alternating{}", self.size);
    }

    fn get_initial_stage(&self) -> Self::Stage {
        return AlternatingStage::Downs;
    }

    fn init_grids(&self) -> (Self::Grids, Self::Surfaces) {
        let grid1 = make_sparse_grid(self.size);
        let grid2 = make_sparse_grid(self.size);
        let across_surfaces = vec![None; self.number_of_clues];
        let down_surfaces = vec![None; self.number_of_clues];
        return ((grid1, grid2), (across_surfaces, down_surfaces));
    }

    fn calculate_number_of_initial_pairs(&self) -> usize {
        let number_of_pairs = self.pairs.len();
        return number_of_pairs * (number_of_pairs - 1);
    }

    fn get_initial_pairs(
        &self,
    ) -> Permutations<std::slice::Iter<(std::string::String, Vec<char>, Vec<char>)>> {
        return self.pairs.iter().permutations(2);
    }

    fn reset_and_place_initial_pairs(
        &self,
        grids: &mut Self::Grids,
        (across_surfaces, down_surfaces): &mut Self::Surfaces,
        pairs: &Vec<&Pair>,
    ) {
        reset_sparse_grid(&mut grids.0);
        reset_sparse_grid(&mut grids.1);

        // reset the surfaces
        for i in 0..self.number_of_clues {
            across_surfaces[i] = None;
        }
        for i in 0..self.number_of_clues {
            down_surfaces[i] = None;
        }

        let (across_surface_1, w11, w12) = pairs[0];
        across_surfaces[0] = Some(across_surface_1.clone());
        let (across_surface_2, w21, w22) = pairs[1];
        across_surfaces[1] = Some(across_surface_2.clone());
        init_sparse_grid(&mut grids.0, w11, w21);
        init_sparse_grid(&mut grids.1, w12, w22);
    }

    fn get_next_pairs(&self, stage: &Self::Stage, grids: &Self::Grids) -> Vec<Vec<&Pair>> {
        match stage {
            AlternatingStage::Downs => {
                return sparse_find_possible_downs(
                    self.size,
                    &self.mask_lookup,
                    &self.empty_vec,
                    &grids.0,
                    &grids.1,
                );
            }
            AlternatingStage::FinalAcrosses => {
                return sparse_find_possible_final_acrosses(
                    self.size,
                    &self.mask_lookup,
                    &grids.0,
                    &grids.1,
                    &self.empty_vec,
                );
            }
        }
    }

    fn place_next_pairs(
        &self,
        stage: &Self::Stage,
        grids: &mut Self::Grids,
        surfaces: &mut Self::Surfaces,
        pairs: &Vec<&Pair>,
    ) -> Option<Self::Stage> {
        match stage {
            AlternatingStage::Downs => {
                sparse_place_down_clues(&mut grids.0, &mut grids.1, &mut surfaces.1, &pairs);
                return Some(AlternatingStage::FinalAcrosses);
            }
            AlternatingStage::FinalAcrosses => {
                sparse_place_final_across_clues(
                    &mut grids.0,
                    &mut grids.1,
                    &mut surfaces.0,
                    &pairs,
                );
                return None;
            }
        }
    }

    fn get_final_statuses(
        &self,
        _grids: &Self::Grids,
        _surfaces: &mut Self::Surfaces,
    ) -> Vec<PairStatus> {
        // all the grids are fully correct by construction
        return vec![];
    }

    fn is_happy(&self, grids: &Self::Grids) -> bool {
        return sparse_no_duplicates_in_grid(self.size, &grids.0, &grids.1);
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

    fn get_all_words(&self, grids: &Self::Grids) -> Vec<Word> {
        let mut words = sparse_get_all_words(self.size, &grids.0);
        let words2 = sparse_get_all_words(self.size, &grids.1);
        words.extend(words2);
        return words;
    }
}
