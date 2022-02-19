use sqlite::Value;
pub mod csv;
pub fn get_connection() -> sqlite::Connection {
    let connection = sqlite::open("./clues.sqlite").unwrap();
    return connection;
}

pub fn ensure_table_exists(connection: &sqlite::Connection) {
    connection
        .execute("CREATE TABLE IF NOT EXISTS clues (crossword_id TEXT, surface TEXT, solution TEXT, UNIQUE(crossword_id, surface, solution));")
        .unwrap();
}

pub fn insert_into_table<'a>(
    connection: &sqlite::Connection,
    crossword_id: &'a str,
    surface: &'a str,
    solution: &'a str,
) -> Option<()> {
    let mut statement = connection
        .prepare("INSERT INTO clues (crossword_id, surface, solution) VALUES (?,?,?);")
        .unwrap();
    statement.bind(1, &Value::String(crossword_id.to_string()));
    statement.bind(2, &Value::String(surface.to_string()));
    statement.bind(3, &Value::String(solution.to_string()));
    statement.next().ok()?;
    return Some(());
}
