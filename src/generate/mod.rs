use combinations::Combinations;
use itertools::Itertools;
use std::collections::{HashMap, HashSet};

fn make_two_letter_lookup(words: &HashSet<Word>) -> HashMap<Word, HashSet<Word>> {
    let mut lookup: HashMap<Word, HashSet<Word>> = HashMap::new();
    for word in words {
        let chars = word.iter().take(2).map(|c| c.clone()).collect();
        let key_words = lookup.entry(chars).or_insert(HashSet::new());
        (*key_words).insert(word.clone());
    }
    return lookup;
}

type Word = Vec<char>;

fn extract_words(multi_surfaces: &HashMap<String, HashSet<String>>) -> HashSet<Word> {
    let mut words = HashSet::new();
    for (key, solutions) in multi_surfaces {
        for solution in solutions {
            let word = solution.clone().chars().collect();
            words.insert(word);
        }
    }
    return words;
}

type Grid = Vec<Vec<char>>;

fn make_empty_grid() -> Grid {
    return vec![
        vec!['.', '.', '.', '.'],
        vec!['.', '.', '.', '.'],
        vec!['.', '.', '.', '.'],
        vec!['.', '.', '.', '.'],
    ];
}

fn place_word_in_row(grid: &Grid, row: usize, word: &Word) -> Grid {
    let mut g = grid.clone();
    g[row] = word.clone();
    return g;
}

fn place_word_in_col(grid: &Grid, col: usize, word: &Word) -> Grid {
    let mut g = grid.clone();
    for row in 0..word.len() {
        g[row][col] = word[row];
    }
    return g;
}

fn place_top_two_across(grid: &Grid, word1: &Word, word2: &Word) -> Grid {
    let g1 = place_word_in_row(grid, 0, word1);
    return place_word_in_row(&g1, 1, word2);
}

fn find_col_prefix(grid: &Grid, col: usize, l: usize) -> Word {
    let mut prefix = vec![];
    for row in 0..l {
        prefix.push(grid[row][col])
    }
    return prefix;
}

fn find_possible_words_in_col(
    grid: &Grid,
    lookup: &HashMap<Word, HashSet<Word>>,
    col: usize,
    l: usize,
) -> HashSet<Word> {
    let prefix = find_col_prefix(grid, col, l);
    return lookup
        .get(&prefix)
        .map(|ws| ws.clone())
        .unwrap_or(HashSet::new());
}

fn place_down_clues(grid: &Grid, lookup: &HashMap<Word, HashSet<Word>>) -> Vec<Grid> {
    let d1s = find_possible_words_in_col(grid, lookup, 0, 2);
    let d2s = find_possible_words_in_col(grid, lookup, 1, 2);
    let d3s = find_possible_words_in_col(grid, lookup, 2, 2);
    let d4s = find_possible_words_in_col(grid, lookup, 3, 2);
    let mut grids = vec![];
    for d1 in &d1s {
        for d2 in &d2s {
            for d3 in &d3s {
                for d4 in &d4s {
                    let g = place_word_in_col(grid, 0, &d1);
                    let g = place_word_in_col(&g, 1, &d2);
                    let g = place_word_in_col(&g, 2, &d3);
                    let g = place_word_in_col(&g, 3, &d4);
                    grids.push(g);
                }
            }
        }
    }
    return grids;
}

pub fn print_grid(g: &Grid) {
    for row in g {
        let word: String = row.iter().cloned().collect();
        println!("{}", word);
    }
}

fn get_all_words(g: &Grid) -> Vec<Word> {
    let a1 = get_word_in_row(g, 0);
    let a2 = get_word_in_row(g, 1);
    let a3 = get_word_in_row(g, 2);
    let a4 = get_word_in_row(g, 3);

    let d1 = get_word_in_col(g, 0);
    let d2 = get_word_in_col(g, 1);
    let d3 = get_word_in_col(g, 2);
    let d4 = get_word_in_col(g, 3);

    return vec![a1, a2, a3, a4, d1, d2, d3, d4];
}

pub fn has_no_duplicates(grid: &Grid) -> bool {
    let words = get_all_words(grid);
    let words_set: HashSet<Word> = words.iter().cloned().collect();
    return words.len() == words_set.len();
}

pub fn make_grids(multi_surfaces: &HashMap<String, HashSet<String>>) -> Vec<Grid> {
    let words = extract_words(&multi_surfaces);
    println!("There are {} words", words.len());
    println!("There are {} multi-surfaces", multi_surfaces.len());
    let lookup = make_two_letter_lookup(&words);
    let grid = make_empty_grid();
    let words_vec = words.iter().collect();
    let mut full_grids = vec![];
    for pair in Combinations::new(words_vec, 2) {
        let w1 = pair[0];
        let w2 = pair[1];
        let g = place_top_two_across(&grid, w1, w2);
        for full_grid in place_down_clues(&g, &lookup) {
            let w3 = get_word_in_row(&full_grid, 2);
            let w4 = get_word_in_row(&full_grid, 3);
            if words.contains(&w3) && words.contains(&w4) && has_no_duplicates(&g) {
                full_grids.push(full_grid);
            }
        }
    }
    return full_grids;
}

fn get_word_in_row(grid: &Grid, row: usize) -> Word {
    return grid[row].clone();
}

fn get_word_in_col(grid: &Grid, col: usize) -> Word {
    let mut word = vec![];
    for row in 0..grid.len() {
        let c = grid[row][col];
        word.push(c);
    }
    return word;
}

/// hello
pub fn make_pairs_to_surfaces(
    multi_surfaces: &HashMap<String, HashSet<String>>,
) -> HashMap<(Word, Word), String> {
    let mut map = HashMap::new();
    for (surface, solutions) in multi_surfaces {
        let solutions_vec: Vec<&String> = solutions.iter().collect();
        // dbg!(&solutions_vec);
        for pair in solutions_vec.iter().combinations(2) {
            let w1: Word = pair[0].chars().collect();
            let w2: Word = pair[1].chars().collect();
            let p1 = (w1.clone(), w2.clone());
            let p2 = (w2.clone(), w1.clone());
            map.insert(p1, surface.clone());
            map.insert(p2, surface.clone());
        }
    }

    return map;
}

pub fn extract_solution_pairs(g1: &Grid, g2: &Grid) -> Vec<(Word, Word)> {
    let a1 = (get_word_in_row(g1, 0), get_word_in_row(g2, 0));
    let a2 = (get_word_in_row(g1, 1), get_word_in_row(g2, 1));
    let a3 = (get_word_in_row(g1, 2), get_word_in_row(g2, 2));
    let a4 = (get_word_in_row(g1, 3), get_word_in_row(g2, 3));

    let d1 = (get_word_in_col(g1, 0), get_word_in_col(g2, 0));
    let d2 = (get_word_in_col(g1, 1), get_word_in_col(g2, 1));
    let d3 = (get_word_in_col(g1, 2), get_word_in_col(g2, 2));
    let d4 = (get_word_in_col(g1, 3), get_word_in_col(g2, 3));

    return vec![a1, a2, a3, a4, d1, d2, d3, d4];
}
