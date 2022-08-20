//! Fetch clues from the internet

use std::fs::File;

use chrono::Datelike;
use chrono::Duration;
use chrono::Utc;
use clap::Parser;

use crate::crosswords::sources::guardian;
use crate::crosswords::sources::nyt;
use crate::store::ensure_table_exists;
use crate::store::get_connection;
use crate::store::insert_into_table;

fn insert_crossword(
    connection: &sqlite::Connection,
    maybe_clues: Option<Vec<(String, String)>>,
    id: &str,
) -> Option<()> {
    print!("{id}: ");
    match maybe_clues {
        None => println!("0 clues added"),
        Some(clues) => {
            for (surface, solution) in &clues {
                insert_into_table(connection, id, surface.as_str(), solution.as_str())?;
            }
            println!("{} clues added", clues.len());
        }
    }
    return Some(());
}

fn fetch_guardian_clues(connection: &sqlite::Connection) {
    for id in (0..16157).rev() {
        let id_str = format!("{id}");
        let maybe_clues = guardian::get_clues(&id_str);
        let db_id_str = format!("guardian:{id}");
        insert_crossword(connection, maybe_clues, &db_id_str);
    }
}

fn nyt_dates() -> Vec<String> {
    let now = Utc::now();
    let number_of_days = 50000;
    let suggestions = (0..number_of_days)
        .map(|delta| {
            let days_delta = Duration::days(-delta);
            let date = now + days_delta;
            let date_str = format!("{}-{}-{}", date.month(), date.day(), date.year());
            return date_str;
        })
        .collect();
    return suggestions;
}

fn fetch_nyt_clues(connection: &sqlite::Connection) {
    for id in nyt_dates() {
        let id_str = format!("{id}");
        let maybe_clues = nyt::get_clues(&id_str);
        let db_id_str = format!("nyt:{id}");
        insert_crossword(connection, maybe_clues, &db_id_str);
    }
}

fn import_xds(connection: &sqlite::Connection) {
    let file_path = "./data/xd/clues.tsv";
    let file = File::open(file_path).unwrap();
    let mut rdr = csv::ReaderBuilder::new().delimiter(b'\t').from_reader(file);
    for result in rdr.records() {
        let record = result.unwrap();
        let solution = record[2].to_string();
        let surface = record[3].to_string();
        if solution == "" || surface == "" {
            continue;
        }
        insert_into_table(connection, "xd-corpus", surface.as_str(), solution.as_str());
    }
}

/// Fetch clues
#[derive(Parser)]
pub struct Args {
    /// Exclude guardian
    #[clap(long)]
    exclude_guardian: bool,

    /// Exclude guardian
    #[clap(long)]
    exclude_nyt: bool,
}

/// Fetch clues from the internet
pub fn run(args: Args) {
    let connection = get_connection();
    ensure_table_exists(&connection);
    if !args.exclude_guardian {
        fetch_guardian_clues(&connection);
    }
    if !args.exclude_nyt {
        fetch_nyt_clues(&connection);
    }
    import_xds(&connection);
}
