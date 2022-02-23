use super::data::Word;

pub type Grid = Vec<Vec<char>>;

pub fn make_empty_grid(size: usize) -> Grid {
    let row = itertools::repeat_n('.', size).collect();
    return itertools::repeat_n(row, size).collect();
}

pub fn reset_grid(g: &mut Grid) {
    for row in 0..g.len() {
        for col in 0..g[row].len() {
            g[row][col] = '.';
        }
    }
}
pub fn place_word_in_row_mut(grid: &mut Grid, row: usize, word: &Word) {
    grid[row] = word.clone()
}
pub fn place_word_in_col_mut(grid: &mut Grid, col: usize, word: &Word) {
    for row in 0..word.len() {
        grid[row][col] = word[row];
    }
}
pub fn find_col_prefix(grid: &Grid, col: usize, l: usize) -> Word {
    let mut prefix = vec![];
    for row in 0..l {
        prefix.push(grid[row][col])
    }
    return prefix;
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
pub fn get_words_in_row_after(grid: &Grid, after: usize) -> Vec<Word> {
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
pub fn init_grid(g: &mut Grid, w1: &Word, w2: &Word) {
    place_word_in_row_mut(g, 0, w1);
    place_word_in_row_mut(g, 1, w2);
}
