//! manipulate grids

use super::data::Word;

/// A grid to hold a crossword puzzle
pub type Grid = Vec<Vec<char>>;


/// Make an empty sparse grid - this is a 5x5 grid with four clues around the edges
/// and two crossing in the middle:
///  S I M O N
///  I . A . O   
///  M A X I M
///  O . I . I
///  N O M I S
pub fn make_sparse_grid() -> Grid {
    let full_row: Vec<char> = itertools::repeat_n('.', 5).collect();
    let sparse_row = vec!['.',  '#',  '.', '#', '.' ];
    // return itertools::repeat_n(row, size).collect();
    return vec![full_row.clone(), 
    sparse_row.clone(), 
    full_row.clone(), 
    sparse_row.clone(), 
    full_row.clone() ];
}

/// Reset all the values in the grid
pub fn reset_sparse_grid(g: &mut Grid) {
    for row in 0..g.len() {
        for col in 0..g[row].len() {
            g[row][col] = '.';
        }
    }
    g[1][1] = '#';
    g[1][3] = '#';
    g[3][1] = '#';
    g[3][3] = '#';
}

/// Place a word in a row in place
pub fn sparse_place_word_in_row_mut(grid: &mut Grid, row: usize, word: &Word) {
    assert!(row != 1);
    assert!(row != 3);
    grid[row] = word.clone()
}

/// Place a word in a col in place
pub fn sparse_place_word_in_col_mut(grid: &mut Grid, col: usize, word: &Word) {
    assert!(col != 1);
    assert!(col != 3);
    for row in 0..word.len() {
        grid[row][col] = word[row];
    }
}

/// Initialise two grids with a word for each
pub fn init_sparse_grid(g: &mut Grid, w1: &Word, w2: &Word) {
    sparse_place_word_in_row_mut(g, 0, w1);
    sparse_place_word_in_row_mut(g, 2, w2);
}


///original functions below

/// Make an empty grid of the given size
pub fn make_empty_grid(size: usize) -> Grid {
    let row = itertools::repeat_n('.', size).collect();
    return itertools::repeat_n(row, size).collect();
}

/// Reset all the values in the grid
pub fn reset_grid(g: &mut Grid) {
    for row in 0..g.len() {
        for col in 0..g[row].len() {
            g[row][col] = '.';
        }
    }
}

/// Place a word in a row in place
pub fn place_word_in_row_mut(grid: &mut Grid, row: usize, word: &Word) {
    grid[row] = word.clone()
}

/// Place a word in a col in place
pub fn place_word_in_col_mut(grid: &mut Grid, col: usize, word: &Word) {
    for row in 0..word.len() {
        grid[row][col] = word[row];
    }
}

/// Get the prefix of size `l` in the given column
pub fn find_col_prefix(grid: &Grid, col: usize, l: usize) -> Word {
    let mut prefix = vec![];
    for row in 0..l {
        prefix.push(grid[row][col])
    }
    return prefix;
}

/// Get the mask specified in the given column
pub fn find_col_mask(grid: &Grid, col: usize,  mask: Vec<usize>) -> Word {
    let mut prefix = vec![];
    for row in mask {
        prefix.push(grid[row][col])
    }
    return prefix;
}

/// Get the mask specified in the given row
pub fn find_row_mask(grid: &Grid, row: usize,  mask: Vec<usize>) -> Word {
    let mut prefix = vec![];
    for col in mask {
        prefix.push(grid[row][col])
    }
    return prefix;
}

/// Print the grid
pub fn print_grid(g: &Grid) {
    for row in g {
        let word: String = row.iter().cloned().collect();
        println!("{}", word);
    }
}

/// Get all the words in the grid (across and down)
pub fn get_all_words(g: &Grid) -> Vec<Word> {
    let size = g.len();
    let mut words: Vec<Word> = (0..size)
        .map(|row| {
            return get_word_in_row(g, row).clone();
        })
        .collect();
    let mut downs: Vec<Word> = (0..size)
        .map(|col| {
            return get_word_in_col(g, col);
        })
        .collect();
    words.append(&mut downs);
    return words;
}

/// Get all the words in the grid (across and down)
pub fn sparse_get_all_words(g: &Grid) -> Vec<Word> {
    let rows = vec![0,2,4];
    let mut words: Vec<Word> = rows.into_iter()
        .map(|row| {
            return get_word_in_row(g, row).clone();
        })
        .collect();
    let cols = vec![0,2,4];
    let mut downs: Vec<Word> = cols.into_iter()
        .map(|col| {
            return get_word_in_col(g, col);
        })
        .collect();
    words.append(&mut downs);
    return words;
}


/// Get all the across words in the grid after (and not
/// including) the given row
pub fn get_words_in_row_after(grid: &Grid, after: usize) -> Vec<&Word> {
    let mut words = vec![];
    let mut i = 0;
    for row in grid {
        if i > after {
            words.push(row);
        }
        i += 1;
    }
    return words;
}

/// Get the word in a give row
pub fn get_word_in_row(grid: &Grid, row: usize) -> &Word {
    return &grid[row];
}

/// Get the word in a given column
fn get_word_in_col(grid: &Grid, col: usize) -> Word {
    let mut word = vec![];
    for row in 0..grid.len() {
        let c = grid[row][col];
        word.push(c);
    }
    return word;
}

/// Initialise two grids with a word for each
pub fn init_grid(g: &mut Grid, w1: &Word, w2: &Word) {
    place_word_in_row_mut(g, 0, w1);
    place_word_in_row_mut(g, 1, w2);
}
