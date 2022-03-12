//! CVS reading/writing

use csv::Writer;
use std::fs::File;

use crate::generate::data::Word;

/// Get all the clues from a csv
pub fn get_clues() -> Option<Vec<(String, Word)>> {
    let file_path = "./clues.csv";
    // let file_path = "./shuffled_clues.csv";
    let file = File::open(file_path).ok()?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut clues = vec![];
    for result in rdr.records() {
        // let record = result.ok()?;
        // let surface = record[0]
        //     .to_string()
        //     .trim()
        //     .trim_end_matches(".")
        //     .to_string();
        // let solution_str = record[1].to_string();
        // let solution: Word = solution_str.chars().collect();
        // let clue = (surface, solution);
        // todo this was the code i added when the parsing wasn't working
        match result {
            Ok(record) => {
        // let record = result.ok()?;
        let surface = record[0]
            .to_string()
            .trim()
            .trim_end_matches(".")
            .to_string();
        let solution_str = record[1].to_string();
        let solution: Word = solution_str.chars().collect();
        let clue = (surface, solution);
        clues.push(clue)

            }
            Err(_ ) => {}
            
        }
        // dbg!(&result);
        // let record = result.ok()?;
        // let clue = (record[0].to_string(), record[1].to_string());
        // clues.push(clue)
    }
    return Some(clues);
}

/// Save all the clues in a CSV
pub fn write_clues(clues: Vec<(String, String)>) -> Option<()> {
    let file_path = "./clues.csv";
    let mut writer = Writer::from_path(file_path).ok()?;
    writer.write_record(&["surface", "solution"]).ok()?;
    for (surface, solution) in clues {
        writer.write_record(&[surface, solution]).ok()?;
    }
    writer.flush().ok()?;
    Some(())
}
