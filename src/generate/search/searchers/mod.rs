//! A searcher for finding quinian crosswords

pub mod dense;

use itertools::Combinations;

use crate::generate::{data::Word, qc::QuinianCrossword};

/// A surface and its two solutions
pub type Pair = (String, Word, Word);

/// The status of a pair
pub enum PairStatus {
    /// The pair has a surface
    HasSurface(String),
    /// The pair are both words, but don't have a common surface
    Words,
    /// The pair do not have words
    NotWords,
}

/// A searcher
pub trait Searcher {
    /// The grids
    type Grids;
    /// The sufaces
    type Surfaces;
    /// Any params needed to init the searcher
    type Params;
    /// Make a new Searcher
    fn new(params: Self::Params, clues: Vec<(String, Word)>) -> Self;
    /// Return the crossword type
    fn crossword_type(&self) -> String;
    /// Initialise some grids
    fn init_grids(&self) -> (Self::Grids, Self::Surfaces);
    /// Calculate the number of initial pairs
    fn calculate_number_of_initial_pairs(&self) -> usize;
    /// Get the first pairs to place
    fn get_initial_pairs(
        &self,
    ) -> Combinations<std::slice::Iter<(std::string::String, Vec<char>, Vec<char>)>>;
    /// Reset the grid and place initial pairs
    fn reset_and_place_initial_pairs(
        &self,
        grids: &mut Self::Grids,
        surfaces: &mut Self::Surfaces,
        pairs: &Vec<&Pair>,
    );
    /// Get other pairs
    fn get_other_pairs(&self, grids: &Self::Grids) -> Vec<Vec<&Pair>>;
    /// Place the other pairs
    fn place_other_pairs(
        &self,
        grids: &mut Self::Grids,
        surfaces: &mut Self::Surfaces,
        pairs: &Vec<&Pair>,
    );
    /// Get the final status of the words
    fn get_final_statuses(
        &self,
        grids: &Self::Grids,
        surfaces: &mut Self::Surfaces,
    ) -> Vec<PairStatus>;
    /// Is the final grids good
    fn is_happy(&self, grids: &Self::Grids) -> bool;
    /// Get the final crossword
    fn get_crossword(&self, grids: &Self::Grids, surfaces: &Self::Surfaces) -> QuinianCrossword;
}
