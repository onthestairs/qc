//! Generate all words

pub mod searchers;

use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;
use std::time::Instant;

use searchers::PairStatus;
use searchers::Searcher;

use super::qc::QuinianCrossword;

/// Make a hash of a crossword
pub fn hash_crossword(crossword: &QuinianCrossword) -> u64 {
    let mut hasher = DefaultHasher::new();
    crossword.hash(&mut hasher);
    return hasher.finish();
}

fn find_and_place_pairs<S, F>(
    searcher: &S,
    stage: S::Stage,
    grids: &mut S::Grids,
    surfaces: &mut S::Surfaces,
    allowed_missing_surfaces: usize,
    on_found: &mut F,
) where
    S: Searcher,
    F: FnMut(&QuinianCrossword, String, usize) -> (),
{
    for other_pairs in searcher.get_next_pairs(&stage, &grids) {
        let maybe_next_stage = searcher.place_next_pairs(&stage, grids, surfaces, &other_pairs);
        if let Some(next_stage) = maybe_next_stage {
            find_and_place_pairs(
                searcher,
                next_stage,
                grids,
                surfaces,
                allowed_missing_surfaces,
                on_found,
            );
        } else {
            let final_word_statuses = searcher.get_final_statuses(&grids, surfaces);

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
                let quinian_crossword = searcher.get_crossword(&grids, &surfaces);
                let crossword_type = searcher.crossword_type();
                on_found(&quinian_crossword, crossword_type.clone(), score);
            }
        }
    }
}
/// Find grids using the given searcher
pub fn find_grids_with_searcher<T, F>(
    start_index: usize,
    allowed_missing_surfaces: usize,
    searcher: &T,
    on_found: &mut F,
) where
    T: Searcher,
    F: FnMut(&QuinianCrossword, String, usize) -> (),
{
    let mut i = 0;
    let mut batch_start_time = Instant::now();
    // get all the initial pairs
    let (mut grids, mut surfaces) = searcher.init_grids();
    let number_of_initial_pairs = searcher.calculate_number_of_initial_pairs();
    let initial_pairs = searcher.get_initial_pairs();
    for pairs in initial_pairs {
        i += 1;
        // check if we should be skipping this index
        if i < start_index {
            continue;
        }
        // check if the first word of the first pair is first
        // alphabetically -- otherwise we will be end up duplicating
        // the work when we receive the pair in the opposite direction
        // (this only works if we are placing 2 initial pairs.
        // it should probably actually be up to the searcher to filter
        // these)
        if pairs[0].1 > pairs[0].2 {
            continue;
        }
        searcher.reset_and_place_initial_pairs(&mut grids, &mut surfaces, &pairs);
        let initial_stage = searcher.get_initial_stage();
        find_and_place_pairs(
            searcher,
            initial_stage,
            &mut grids,
            &mut surfaces,
            allowed_missing_surfaces,
            on_found,
        );
        if i % 10000 == 0 {
            let duration = batch_start_time.elapsed();
            let percent = (i as f32 / number_of_initial_pairs as f32) * 100.0;
            println!("Done {i} ({percent}%) in {duration:?}");
            batch_start_time = Instant::now();
        }
    }
}
