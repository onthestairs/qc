//! Generate all words

pub mod searchers;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Instant;

use super::qc::QuinianCrossword;
use searchers::PairStatus;
use searchers::Searcher;

/// Make a hash of a crossword
pub fn hash_crossword(crossword: &QuinianCrossword) -> u64 {
    let mut hasher = DefaultHasher::new();
    crossword.hash(&mut hasher);
    return hasher.finish();
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
    let (mut grids, mut surfaces) = searcher.init_grids();
    let number_of_initial_pairs = searcher.calculate_number_of_initial_pairs();
    let initial_pairs = searcher.get_initial_pairs();
    for pairs in initial_pairs {
        i += 1;
        if i < start_index {
            continue;
        }
        searcher.reset_and_place_initial_pairs(&mut grids, &mut surfaces, &pairs);
        for other_pairs in searcher.get_other_pairs(&grids) {
            searcher.place_other_pairs(&mut grids, &mut surfaces, &other_pairs);
            let final_word_statuses = searcher.get_final_statuses(&grids, &mut surfaces);

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
                on_found(&quinian_crossword, crossword_type.clone(), score);
            }
        }
        if i % 10000 == 0 {
            let duration = batch_start_time.elapsed();
            let percent = (i as f32 / number_of_initial_pairs as f32) * 100.0;
            println!("Done {i} ({percent}%) in {duration:?}");
            batch_start_time = Instant::now();
        }
    }
}
