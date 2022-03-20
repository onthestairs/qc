//! Clue store

use sqlite::{State, Value};

use crate::generate::qc::QuinianCrossword;
use crate::generate::search::hash_crossword;

pub mod csv;
pub mod word_frequencies;

/// Get a connection to an sqlite db
pub fn get_connection() -> sqlite::Connection {
    let connection = sqlite::open("./clues.sqlite").unwrap();
    return connection;
}

/// Ensure that the table exists
pub fn ensure_table_exists(connection: &sqlite::Connection) {
    connection
        .execute("CREATE TABLE IF NOT EXISTS clues (crossword_id TEXT, surface TEXT, solution TEXT, UNIQUE(crossword_id, surface, solution));")
        .unwrap();
}

/// Insert a clue into the db
pub fn insert_into_table<'a>(
    connection: &sqlite::Connection,
    crossword_id: &'a str,
    surface: &'a str,
    solution: &'a str,
) -> Option<()> {
    let mut statement = connection
        .prepare("INSERT INTO clues (crossword_id, surface, solution) VALUES (?,?,?);")
        .unwrap();
    let _ = statement.bind(1, &Value::String(crossword_id.to_string()));
    let _ = statement.bind(2, &Value::String(surface.to_string()));
    let _ = statement.bind(3, &Value::String(solution.to_string()));
    let result = statement.next();
    match result {
        Err(e) => println!("{e}"),
        Ok(_r) => (),
    }

    return Some(());
}

/// Get all the clues in the db
pub fn select_all_clues(connection: &sqlite::Connection) -> Option<Vec<(String, String)>> {
    let mut clues = vec![];

    let mut statement = connection
        .prepare("SELECT surface, solution FROM clues")
        .unwrap();

    while let State::Row = statement.next().unwrap() {
        let surface = statement.read::<String>(0).ok()?;
        let solution = statement.read::<String>(1).ok()?;
        let clue = (surface, solution);
        clues.push(clue);
    }
    return Some(clues);
}

/// Ensure that the table exists
pub fn ensure_results_table_exists(connection: &sqlite::Connection) {
    connection
        .execute("CREATE TABLE IF NOT EXISTS results (crossword TEXT, hash INT, crossword_type TEXT, score INT, UNIQUE(hash));")
        .unwrap();
}

/// Insert a result to the database
pub fn insert_result_into_table<'a>(
    connection: &sqlite::Connection,
    crossword: &QuinianCrossword,
    crossword_type: String,
    score: usize,
) -> Option<()> {
    let mut statement = connection
        .prepare("INSERT INTO results (crossword, hash, crossword_type, score) VALUES (?,?,?,?);")
        .unwrap();
    let serialised_crossword = serde_json::to_string(crossword).unwrap();
    let crossword_hash = hash_crossword(crossword);
    let _ = statement.bind(1, &Value::String(serialised_crossword.to_string()));
    let _ = statement.bind(2, &Value::Integer(crossword_hash as i64));
    let _ = statement.bind(3, &Value::String(crossword_type));
    let _ = statement.bind(4, &Value::Integer(score as i64));
    let result = statement.next();
    match result {
        Err(e) => println!("{e}"),
        Ok(_r) => (),
    }

    return Some(());
}

/// Get all the clues in the db
pub fn get_results(
    connection: &sqlite::Connection,
    size: usize,
    score_at_most: usize,
) -> Option<Vec<(QuinianCrossword, u64)>> {
    let mut crosswords = vec![];

    let mut statement = connection
        .prepare("SELECT crossword, score FROM results WHERE size = ? AND score <= ?")
        .unwrap();
    let _ = statement.bind(1, &Value::Integer(size as i64));
    let _ = statement.bind(2, &Value::Integer(score_at_most as i64));

    while let State::Row = statement.next().unwrap() {
        let serialised_crossword = statement.read::<String>(0).ok()?;
        let crossword: QuinianCrossword = serde_json::from_str(&serialised_crossword).ok()?;
        let score = statement.read::<i64>(1).ok()?;
        crosswords.push((crossword, score as u64));
    }
    return Some(crosswords);
}
