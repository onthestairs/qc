use qc::store::csv::write_clues;
use qc::store::get_connection;
use qc::store::select_all_clues;

fn main() {
    let connection = get_connection();
    let clues = select_all_clues(&connection).unwrap();
    write_clues(clues);
}
