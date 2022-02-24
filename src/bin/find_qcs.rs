use clap::Parser;
use qc::generate::search::{find_solutions, print_solution};
use qc::store::csv::get_clues;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Size of the grid
    #[clap(short, long, default_value_t = 3)]
    size: usize,

    /// Number of allowed non-surface words
    #[clap(short, long, default_value_t = 0)]
    allowed_missing_surfaces: usize,
}

fn main() {
    let args = Args::parse();

    let clues = get_clues().unwrap();
    let solutions = find_solutions(args.size, args.allowed_missing_surfaces, clues);
    for solution in solutions {
        print_solution(&solution);
        println!("~~~~~~~~");
        println!("");
        println!("");
    }
    // dbg!(solutions);
}
