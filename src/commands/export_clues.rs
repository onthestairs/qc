//! Export the clues from the database into a CSV file

use clap::Parser;

use crate::store::csv::write_clues;
use crate::store::get_connection;
use crate::store::select_all_clues;

/// Export clues
#[derive(Parser)]
pub struct Args {}

/// Export clues from the database into a csv
pub fn run(_args: Args) {
    let connection = get_connection();
    let clues = select_all_clues(&connection).unwrap();
    write_clues(clues);
}
