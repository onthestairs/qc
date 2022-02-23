use qc::generate::search::{find_solutions, print_solution};
use qc::store::csv::get_clues;

fn main() {
    let clues = get_clues().unwrap();
    let solutions = find_solutions(4, 2, clues);
    for solution in solutions {
        print_solution(&solution);
        println!("~~~~~~~~");
        println!("");
        println!("");
    }
    // dbg!(solutions);
}
