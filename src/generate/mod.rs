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

pub type Word = Vec<char>;

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

pub type Grid = Vec<Vec<char>>;

pub fn make_empty_grid(size: usize) -> Grid {
    let row = itertools::repeat_n('.', size).collect();
    return itertools::repeat_n(row, size).collect();
    // return vec![
    //     vec!['.', '.', '.', '.'],
    //     vec!['.', '.', '.', '.'],
    //     vec!['.', '.', '.', '.'],
    //     vec!['.', '.', '.', '.'],
    // ];
}

pub fn reset_grid(g: &mut Grid) {
    for row in 0..g.len() {
        for col in 0..g[row].len() {
            g[row][col] = '.';
        }
    }
}

fn place_word_in_row(grid: &Grid, row: usize, word: &Word) -> Grid {
    let mut g = grid.clone();
    g[row] = word.clone();
    return g;
}

pub fn place_word_in_row_mut(grid: &mut Grid, row: usize, word: &Word) {
    grid[row] = word.clone()
}

fn place_word_in_col(grid: &Grid, col: usize, word: &Word) -> Grid {
    let mut g = grid.clone();
    for row in 0..word.len() {
        g[row][col] = word[row];
    }
    return g;
}

pub fn place_word_in_col_mut(grid: &mut Grid, col: usize, word: &Word) {
    for row in 0..word.len() {
        grid[row][col] = word[row];
    }
}

fn place_top_two_across(grid: &Grid, word1: &Word, word2: &Word) -> Grid {
    let g1 = place_word_in_row(grid, 0, word1);
    return place_word_in_row(&g1, 1, word2);
}

pub fn find_col_prefix(grid: &Grid, col: usize, l: usize) -> Word {
    let mut prefix = vec![];
    // dbg!(grid);
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
    let size = grid.len();
    let ds: Vec<HashSet<Word>> = (0..size)
        .map(|col| {
            return find_possible_words_in_col(grid, lookup, col, 2);
        })
        .collect();
    if ds.iter().any(|d| d.len() == 0) {
        return vec![];
    }
    let wss = ds.iter().multi_cartesian_product();
    let mut grids = vec![];
    for ws in wss {
        let mut g = grid.clone();
        for col in 0..size {
            g = place_word_in_col(&g, col, &ws[col]);
        }
        grids.push(g);
    }
    return grids;
}

pub fn print_grid(g: &Grid) {
    for row in g {
        let word: String = row.iter().cloned().collect();
        println!("{}", word);
    }
}

pub fn get_all_words(g: &Grid) -> Vec<Word> {
    let size = g.len();
    let mut words: Vec<Word> = (0..size)
        .map(|row| {
            return get_word_in_row(g, row);
        })
        .collect();
    let downs: Vec<Word> = (0..size)
        .map(|col| {
            return get_word_in_col(g, col);
        })
        .collect();
    words.extend(downs);
    return words;
}

pub fn has_no_duplicates(grid: &Grid) -> bool {
    let words = get_all_words(grid);
    let words_set: HashSet<Word> = words.iter().cloned().collect();
    return words.len() == words_set.len();
}

pub fn has_no_duplicates_2(g1: &Grid, g2: &Grid) -> bool {
    let mut words = get_all_words(g1);
    let ws2 = get_all_words(g2);
    words.extend(ws2);

    let words_set: HashSet<Word> = words.iter().cloned().collect();
    return words.len() == words_set.len();
}

fn get_words_in_row_after(grid: &Grid, after: usize) -> Vec<Word> {
    let mut words = vec![];
    let mut i = 0;
    for row in grid {
        if i > after {
            words.push(row.clone());
        }
        i += 1;
    }
    return words;
}

pub fn make_grids(size: usize, multi_surfaces: &HashMap<String, HashSet<String>>) -> Vec<Grid> {
    let words = extract_words(&multi_surfaces);
    println!("There are {} words", words.len());
    println!("There are {} multi-surfaces", multi_surfaces.len());
    let lookup = make_two_letter_lookup(&words);
    let grid = make_empty_grid(size);
    let words_vec = words.iter().collect();
    // let mut full_grids: HashSet<Grid> = HashSet::new();
    let mut full_grids = vec![];
    for pair in Combinations::new(words_vec, 2) {
        let w1 = pair[0];
        let w2 = pair[1];
        let g = place_top_two_across(&grid, w1, w2);
        for full_grid in place_down_clues(&g, &lookup) {
            // print_grid(&full_grid);
            let ws = get_words_in_row_after(&full_grid, 1);
            let has_real_words = ws.into_iter().all(|w| words.contains(&w));
            if has_real_words && has_no_duplicates(&full_grid) {
                full_grids.push(full_grid);
            }
        }
    }
    return full_grids;
}

pub fn get_word_in_row(grid: &Grid, row: usize) -> Word {
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
    let g1_words = get_all_words(g1);
    let g2_words = get_all_words(g2);
    return g1_words.into_iter().zip(g2_words.into_iter()).collect();
    //
    // let a1 = (get_word_in_row(g1, 0), get_word_in_row(g2, 0));
    // let a2 = (get_word_in_row(g1, 1), get_word_in_row(g2, 1));
    // let a3 = (get_word_in_row(g1, 2), get_word_in_row(g2, 2));
    // let a4 = (get_word_in_row(g1, 3), get_word_in_row(g2, 3));
    //
    // let d1 = (get_word_in_col(g1, 0), get_word_in_col(g2, 0));
    // let d2 = (get_word_in_col(g1, 1), get_word_in_col(g2, 1));
    // let d3 = (get_word_in_col(g1, 2), get_word_in_col(g2, 2));
    // let d4 = (get_word_in_col(g1, 3), get_word_in_col(g2, 3));
    //
    // return vec![a1, a2, a3, a4, d1, d2, d3, d4];
}
