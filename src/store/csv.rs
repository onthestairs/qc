use csv::Writer;
use std::fs::File;

pub fn get_clues() -> Option<Vec<(String, String)>> {
    let file_path = "./clues.csv";
    let file = File::open(file_path).ok()?;
    let mut rdr = csv::Reader::from_reader(file);
    let mut clues = vec![];
    for result in rdr.records() {
        let record = result.ok()?;
        let clue = (record[0].to_string(), record[1].to_string());
        clues.push(clue)
    }
    return Some(clues);
}

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
