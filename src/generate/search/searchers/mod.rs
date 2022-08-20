//! A searcher for finding quinian crosswords

pub mod alternating;
pub mod dense;

use itertools::Combinations;

use crate::generate::data::Surface;
use crate::generate::data::Word;
use crate::generate::qc::QuinianCrossword;

/// A surface and its two solutions
pub type Pair = (Surface, Word, Word);

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
    /// The stages
    type Stage;
    /// Make a new Searcher
    fn new(params: Self::Params, clues: Vec<(Surface, Word)>) -> Self;
    /// Return the crossword type
    fn crossword_type(&self) -> String;
    /// The first stage
    fn get_initial_stage(&self) -> Self::Stage;
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
    fn get_next_pairs(&self, stage: &Self::Stage, grids: &Self::Grids) -> Vec<Vec<&Pair>>;
    /// Place the next pairs and return the next stage
    fn place_next_pairs(
        &self,
        stage: &Self::Stage,
        grids: &mut Self::Grids,
        surfaces: &mut Self::Surfaces,
        pairs: &Vec<&Pair>,
    ) -> Option<Self::Stage>;
    /// Get the final status of the words
    fn get_final_statuses(
        &self,
        grids: &Self::Grids,
        surfaces: &mut Self::Surfaces,
    ) -> Vec<PairStatus>;
    /// Is the final grid good
    fn is_happy(&self, grids: &Self::Grids) -> bool;
    /// Get the final crossword
    fn get_crossword(&self, grids: &Self::Grids, surfaces: &Self::Surfaces) -> QuinianCrossword;
}
