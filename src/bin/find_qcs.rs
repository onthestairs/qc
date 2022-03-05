use clap::Parser;
use qc::generate::qc::QuinianCrossword;
use qc::generate::search::find_solutions;
use qc::store::csv::get_clues;
use qc::store::{ensure_results_table_exists, get_connection, insert_result_into_table};

/// Program to generate quinian crosswords
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Size of the grid
    #[clap(short, long, default_value_t = 3)]
    size: usize,

    /// Number of allowed non-surface words
    #[clap(short, long, default_value_t = 0)]
    allowed_missing_surfaces: usize,

    /// Start index
    #[clap(long, default_value_t = 0)]
    start_index: usize,
}

fn main() {
    let args = Args::parse();

    let connection = get_connection();
    ensure_results_table_exists(&connection);
    let clues = get_clues().unwrap();
    let save_crossword = |crossword: &QuinianCrossword, size: usize, score: usize| {
        insert_result_into_table(&connection, crossword, size, score);
    };
    find_solutions(
        args.size,
        args.allowed_missing_surfaces,
        args.start_index,
        clues,
        save_crossword,
    );
}
