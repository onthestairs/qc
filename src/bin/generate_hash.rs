use itertools::Itertools;
use std::collections::{HashMap, HashSet};

use qc::generate::extract_solution_pairs;
use qc::generate::get_all_words;
use qc::generate::make_grids;
use qc::generate::make_pairs_to_surfaces;
use qc::generate::print_grid;
use qc::generate::Grid;
use qc::generate::Word;
use qc::munge::get_multi_surfaces;
use qc::store::csv::get_clues;

fn make_solution_to_surfaces(
    multi_surfaces: &HashMap<String, HashSet<String>>,
) -> HashMap<Word, HashSet<String>> {
    let mut lookup: HashMap<Word, HashSet<String>> = HashMap::new();
    for (surface, solutions) in multi_surfaces {
        for solution in solutions {
            let word = solution.clone().chars().collect();
            let solution_surfaces = lookup.entry(word).or_insert(HashSet::new());
            (*solution_surfaces).insert(surface.clone());
        }
    }
    return lookup;
}

fn make_grid_hashes(
    grid: &Grid,
    solution_to_surfaces: &HashMap<Word, HashSet<String>>,
) -> Vec<String> {
    let words = get_all_words(grid);
    let ds: Vec<HashSet<String>> = words
        .into_iter()
        .map(|word| {
            return solution_to_surfaces
                .get(&word)
                .cloned()
                .unwrap_or(HashSet::new());
        })
        .collect();
    let sss = ds.iter().multi_cartesian_product();
    let mut hashes = vec![];
    for ss in sss {
        let hash = ss.into_iter().join("\n");
        hashes.push(hash);
    }
    return hashes;
}

fn all_distinct_words(g1: &Grid, g2: &Grid) -> bool {
    let w1s: HashSet<Word> = get_all_words(g1).into_iter().collect();
    let w2s: HashSet<Word> = get_all_words(g2).into_iter().collect();
    return w1s.intersection(&w2s).count() == 0;
}

fn main() {
    let clues = get_clues().unwrap();
    let size = 3;
    let multi_surfaces = get_multi_surfaces(clues, size);
    let grids = make_grids(size, &multi_surfaces);
    println!("Found {} full grids", grids.len());
    let solution_to_surfaces = make_solution_to_surfaces(&multi_surfaces);
    let mut hash_to_grid: HashMap<String, HashSet<&Grid>> = HashMap::new();
    let mut matches = vec![];
    let mut i = 0;
    for grid in &grids {
        let hashes = make_grid_hashes(&grid, &solution_to_surfaces);
        for hash in hashes {
            let hash_grids = hash_to_grid.entry(hash).or_insert(HashSet::new());
            for hash_grid in hash_grids.iter() {
                if all_distinct_words(grid, hash_grid) {
                    println!("Found match!");
                    print_grid(grid);
                    println!("------");
                    print_grid(hash_grid);
                    println!("~~~~");
                    matches.push((grid.clone(), hash_grid.clone()));
                }
            }
            (*hash_grids).insert(&grid);
        }
        if i % 1000 == 0 {
            println!("Done {i}");
        }
        i += 1;
    }
}
