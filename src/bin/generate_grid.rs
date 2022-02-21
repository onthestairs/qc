use std::collections::{HashMap, HashSet};

use qc::generate::extract_solution_pairs;
use qc::generate::make_grids;
use qc::generate::make_pairs_to_surfaces;
use qc::generate::print_grid;
use qc::munge::get_multi_surfaces;
use qc::store::csv::get_clues;

fn main() {
    let clues = get_clues().unwrap();
    let size = 4;
    let multi_surfaces = get_multi_surfaces(clues, size);
    let grids = make_grids(size, &multi_surfaces);
    println!("Found {} full grids", grids.len());
    let pairs_lookup = make_pairs_to_surfaces(&multi_surfaces);
    let mut best_score = 0;
    for grid_pair in combinations::Combinations::new(grids, 2) {
        let g1 = &grid_pair[0];
        let g2 = &grid_pair[1];
        let solution_pairs = extract_solution_pairs(&g1, &g2);
        let mut score = 0;
        for pair in solution_pairs {
            if pairs_lookup.contains_key(&pair) {
                score += 1;
            }
        }
        if score > best_score {
            best_score = score;
            println!("new best score: {best_score}");
            print_grid(&g1);
            println!("~~~~~~");
            print_grid(&g2);
            println!("=======")
        }
    }
    println!("Best score is {best_score}")
}
