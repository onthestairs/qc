//! Guardian crosswords

use regex::Regex;
use scraper::{Html, Selector};

fn make_url<'a>(id: &'a str) -> String {
    return format!("https://www.theguardian.com/crosswords/quick/{}", id);
}

fn fetch_crossword_html<'a>(id: &'a str) -> Option<String> {
    let url = make_url(id);
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(url.as_str())
        .header("Accept-Language", "en-GB,en")
        .send()
        .ok()?;
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return None;
    }
    let body = response.text().ok()?;
    return Some(body);
}

/// Get the crossword of the given type and id
pub fn fetch_crossword_json<'a>(id: &'a str) -> Option<serde_json::Value> {
    let html = fetch_crossword_html(&id)?;
    let document = Html::parse_document(html.as_str());
    let selector = Selector::parse(".js-crossword").ok()?;
    let element = document.select(&selector).nth(0)?;
    let escaped_json = element.value().attr("data-crossword-data")?;
    let parsed_json = serde_json::from_str(&escaped_json).ok()?;
    return Some(parsed_json);
}

fn strip_enum<'a>(s: &'a str) -> String {
    let re = Regex::new(r"\([0-9,]+\)$").unwrap();
    return re.replace(s, "").to_string();
}

pub fn get_clues<'a>(id: &'a str) -> Option<Vec<(String, String)>> {
    let crossword_json = fetch_crossword_json(id)?;
    return crossword_json
        .get("entries")
        .and_then(|entries| entries.as_array())
        .unwrap_or(&vec![])
        .iter()
        .map(|entry| {
            let surface = entry
                .get("clue")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string())
                .as_ref()
                .map(|s| {
                    return strip_enum(s).trim().to_string();
                })?;
            let answer = entry
                .get("solution")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string())?;
            return Some((surface, answer));
        })
        .collect();
}
