use std::collections::{HashMap, HashSet};

use qc::generate::find_col_prefix;
use qc::generate::get_word_in_row;
use qc::generate::get_words_in_row_after;
use qc::generate::has_no_duplicates;
use qc::generate::has_no_duplicates_2;
use qc::generate::make_empty_grid;
use qc::generate::place_word_in_col_mut;
use qc::generate::place_word_in_row_mut;
use qc::generate::print_grid;
use qc::generate::reset_grid;
use qc::generate::Grid;
use qc::generate::Word;
use qc::munge::get_multi_surfaces;
use qc::store::csv::get_clues;

use itertools::Itertools;
fn make_ms_pairs(multi_surfaces: &HashMap<String, HashSet<String>>) -> Vec<(String, Word, Word)> {
    let mut pairs = vec![];
    for (surface, solutions) in multi_surfaces {
        let solutions_vec: Vec<&String> = solutions.iter().collect();
        for pair in solutions_vec.iter().combinations(2) {
            let w1: Word = pair[0].chars().collect();
            let w2: Word = pair[1].chars().collect();
            if w1 == w2 {
                continue;
            }
            let ms_pair = (surface.clone(), w1.clone(), w2.clone());
            pairs.push(ms_pair);
            let ms_pair2 = (surface.clone(), w2.clone(), w1.clone());
            pairs.push(ms_pair2);
        }
    }
    return pairs;
}

fn make_double_prefix_lookup(
    pairs: &Vec<(String, Word, Word)>,
) -> HashMap<(Word, Word), Vec<(String, Word, Word)>> {
    let mut lookup: HashMap<(Word, Word), Vec<(String, Word, Word)>> = HashMap::new();
    for (s, w1, w2) in pairs {
        let prefix1 = w1.iter().take(2).map(|c| c.clone()).collect();
        let prefix2 = w2.iter().take(2).map(|c| c.clone()).collect();
        let key_words = lookup.entry((prefix1, prefix2)).or_insert(vec![]);
        (*key_words).push((s.clone(), w1.clone(), w2.clone()));
    }
    return lookup;
}

type DoublePrefixLookup = HashMap<(Word, Word), Vec<(String, Word, Word)>>;

fn find_possible_downs(
    lookup: &DoublePrefixLookup,
    grid1: &Grid,
    grid2: &Grid,
) -> Vec<Vec<(String, Word, Word)>> {
    let size = grid1.len();
    let ds: Vec<Vec<(String, Word, Word)>> = (0..size)
        .map(|col| {
            let prefix1 = find_col_prefix(grid1, col, 2);
            let prefix2 = find_col_prefix(grid2, col, 2);
            return lookup.get(&(prefix1, prefix2)).unwrap_or(&vec![]).clone();
        })
        // .cloned()
        .collect();
    if ds.iter().any(|d| d.len() == 0) {
        return vec![];
    }
    return ds.into_iter().multi_cartesian_product().clone().collect();
}

fn init_grid(g: &mut Grid, w1: &Word, w2: &Word) {
    place_word_in_row_mut(g, 0, w1);
    place_word_in_row_mut(g, 1, w2);
}

fn place_down_clues(g1: &mut Grid, g2: &mut Grid, down_combos: Vec<(String, Word, Word)>) {
    let mut col = 0;
    // print_grid(g1);
    // print_grid(g2);
    // dbg!(&down_combos);
    for (surface, w1, w2) in down_combos {
        place_word_in_col_mut(g1, col, &w1);
        place_word_in_col_mut(g2, col, &w2);
        col += 1;
    }
}

fn find_grids(
    size: usize,
    pairs: Vec<(String, Word, Word)>,
    prefix_lookup: DoublePrefixLookup,
    pairs_to_surface: HashMap<(Word, Word), String>,
    word_list: HashSet<Word>,
) -> Vec<(Grid, Grid, Vec<String>)> {
    let mut g1 = make_empty_grid(size);
    let mut g2 = make_empty_grid(size);
    let mut solutions = vec![];
    let number_of_combos = (pairs.len() * (pairs.len() - 1)) / 2;
    println!("Found {} combos", number_of_combos);
    let mut i = 0;
    for pair in pairs.iter().combinations(2) {
        let (surface1, w11, w12) = pair[0];
        let (surface2, w21, w22) = pair[1];
        reset_grid(&mut g1);
        reset_grid(&mut g2);
        init_grid(&mut g1, w11, w21);
        init_grid(&mut g2, w21, w22);
        let down_combos = find_possible_downs(&prefix_lookup, &g1, &g2);
        for down_combo in down_combos {
            place_down_clues(&mut g1, &mut g2, down_combo);
            if !has_no_duplicates_2(&g1, &g2) {
                continue;
            }
            let mut final_words = get_words_in_row_after(&g1, 2);
            let fws2 = get_words_in_row_after(&g2, 2);
            final_words.extend(fws2);

            // if word_list.contains(&lw1) && word_list.contains(&lw2) {
            if final_words.iter().all(|word| word_list.contains(word)) {
                // let maybe_final_surface = pairs_to_surface.get(&(lw1, lw2));
                // if let Some(final_surface) = maybe_final_surface {
                println!("Found quint");
                // println!("Final surface: {final_surface}");
                print_grid(&g1);
                println!("~~~~~~~");
                print_grid(&g2);
                println!("=======\n");
                let solution = (g1.clone(), g2.clone(), vec![]);
                solutions.push(solution)
            }
        }
        if i % 10000 == 0 {
            println!("Done {i}");
        }
        i += 1;
    }
    return solutions;
}

fn make_pairs_to_surface(ms_pairs: &Vec<(String, Word, Word)>) -> HashMap<(Word, Word), String> {
    let mut lookup = HashMap::new();
    for (surface, w1, w2) in ms_pairs {
        let key = (w1.clone(), w2.clone());
        lookup.insert(key, surface.clone());
    }
    return lookup;
}

fn make_word_list(ms_pairs: &Vec<(String, Word, Word)>) -> HashSet<Word> {
    let mut words = HashSet::new();
    for (_, w, _) in ms_pairs {
        words.insert(w.clone());
    }
    return words;
}

fn main() {
    let clues = get_clues().unwrap();
    let size = 3;
    let multi_surfaces = get_multi_surfaces(clues, size);
    println!("Found {} multi-surfaces", multi_surfaces.len());
    let ms_pairs = make_ms_pairs(&multi_surfaces);
    let pairs_to_surface = make_pairs_to_surface(&ms_pairs);
    let word_list = make_word_list(&ms_pairs);
    println!("Found {} pairs", ms_pairs.len());
    let double_prefix_lookup = make_double_prefix_lookup(&ms_pairs);
    println!("made double prefix lookup");
    let solutions = find_grids(
        size,
        ms_pairs,
        double_prefix_lookup,
        pairs_to_surface,
        word_list,
    );
    dbg!(solutions);
}
