//! NYT crosswords

use itertools::Itertools;
use scraper::{Html, Selector};

/// Convert a date into one that xwordinfo uses
///
/// The expected format of a date should be MM-DD-YYYY and this will
/// convert the hyphens into slashes
fn make_xwordinfo_date<'a>(date: &'a str) -> String {
    let new_date = date.to_string().replace("-", "/");
    return new_date;
}

fn fetch_crossword_html<'a>(date: &'a str) -> Option<String> {
    let client = reqwest::blocking::Client::new();
    let xwordinfo_date = make_xwordinfo_date(date);
    let response = client
        .get("https://www.xwordinfo.com/Crossword")
        .header(
            "User-agent",
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:74.0) Gecko/20100101 Firefox/74.0",
        )
        .query(&[("date", xwordinfo_date)])
        .send()
        .ok()?;
    let body = response.text().ok()?;
    return Some(body);
}

enum Direction {
    Across,
    Down,
}

fn extract_clues<'a>(document: &'a Html, direction: Direction) -> Option<Vec<(String, String)>> {
    let selector_str = match direction {
        Direction::Across => "#ACluesPan .numclue div",
        Direction::Down => "#DCluesPan .numclue div",
    };
    let clues_selector = Selector::parse(selector_str).ok()?;
    let clue_elements = document.select(&clues_selector);
    let clues = clue_elements
        .tuples()
        .map(|(_number_element, clue_element)| {
            let clue_element_texts: Vec<&'a str> = clue_element.text().collect::<Vec<_>>();
            let surface = clue_element_texts
                .iter()
                .nth(0)?
                .trim_end_matches(" : ")
                .trim()
                .to_string();
            let solution = clue_element_texts.iter().nth(1)?.trim().to_string();
            return Some((surface, solution));
        })
        .collect();
    return clues;
}

/// Get a nyt from the given date
pub fn get_clues<'a>(date: &'a str) -> Option<Vec<(String, String)>> {
    let html = fetch_crossword_html(date)?;
    let document = Html::parse_document(html.as_str());
    let mut clues = extract_clues(&document, Direction::Across)?;
    let down_clues = extract_clues(&document, Direction::Down)?;
    clues.extend(down_clues);
    return Some(clues);
}
