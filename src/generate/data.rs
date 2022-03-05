//! Munge relevant data

use itertools::Itertools;
use std::collections::{HashMap, HashSet};

/// type alias for a word
pub type Word = Vec<char>;

/// Should we include the surface/solution in the analysis
fn should_include(surface: &String, solution: &String, length: usize) -> bool {
    if surface.len() == 0 || surface.starts_with("See") || surface == "<<NO CLUE>>" {
        return false;
    }
    if solution.len() != length {
        return false;
    }
    let is_all_alpha_uppercase = solution.chars().all(|c| char::is_ascii_uppercase(&c));
    if !is_all_alpha_uppercase {
        return false;
    }
    return true;
}

/// find all surfaces with many solutions of length `length`
pub fn get_multi_surfaces(
    clues: &Vec<(String, String)>,
    length: usize,
) -> HashMap<String, HashSet<String>> {
    // make a map from surface to a set of solutions
    let mut surface_solutions: HashMap<String, HashSet<String>> = HashMap::new();
    for (surface, solution) in clues {
        if !should_include(&surface, &solution, length) {
            continue;
        }
        let solutions = surface_solutions
            .entry(surface.clone())
            .or_insert(HashSet::new());
        (*solutions).insert(solution.clone());
    }
    // filter by only those surfaces with many solutions
    return surface_solutions
        .into_iter()
        .filter(|(_, solutions)| solutions.len() >= 2)
        .collect();
}

/// make a map from solution-pairs to the surface they share
pub fn make_pairs_to_surfaces(
    multi_surfaces: &HashMap<String, HashSet<String>>,
) -> HashMap<(Word, Word), String> {
    let mut map = HashMap::new();
    for (surface, solutions) in multi_surfaces {
        let solutions_vec: Vec<&String> = solutions.iter().collect();
        for pair in solutions_vec.iter().combinations(2) {
            let w1: Word = pair[0].chars().collect();
            let w2: Word = pair[1].chars().collect();
            let p1 = (w1.clone(), w2.clone());
            let p2 = (w2.clone(), w1.clone());
            map.insert(p1, surface.clone());
            map.insert(p2, surface.clone());
        }
    }

    return map;
}

/// make a vec of all solution pairs
pub fn make_ms_pairs(
    multi_surfaces: &HashMap<String, HashSet<String>>,
) -> Vec<(String, Word, Word)> {
    let mut pairs = vec![];
    for (surface, solutions) in multi_surfaces {
        let solutions_vec: Vec<&String> = solutions.iter().collect();
        for pair in solutions_vec.iter().combinations(2) {
            let w1: Word = pair[0].chars().collect();
            let w2: Word = pair[1].chars().collect();
            if w1 == w2 {
                continue;
            }
            let ms_pair = (surface.clone(), w1.clone(), w2.clone());
            pairs.push(ms_pair);
            let ms_pair2 = (surface.clone(), w2.clone(), w1.clone());
            pairs.push(ms_pair2);
        }
    }
    return pairs;
}

/// Type for doing a double prefix lookup
pub type PairPrefixLookup = HashMap<(Word, Word), Vec<(String, Word, Word)>>;

/// Make a lookup from the 2-prefix or each word in a pair, to the
/// poissible pairs
pub fn make_pair_prefix_lookup(pairs: &Vec<(String, Word, Word)>) -> PairPrefixLookup {
    let mut lookup: PairPrefixLookup = HashMap::new();
    for (s, w1, w2) in pairs {
        let prefix1 = w1.iter().take(2).map(|c| c.clone()).collect();
        let prefix2 = w2.iter().take(2).map(|c| c.clone()).collect();
        let key_words = lookup.entry((prefix1, prefix2)).or_insert(vec![]);
        (*key_words).push((s.clone(), w1.clone(), w2.clone()));
    }
    return lookup;
}

/// Make a map from a given pair to its surface
pub fn make_pairs_to_surface(
    ms_pairs: &Vec<(String, Word, Word)>,
) -> HashMap<(Word, Word), String> {
    let mut lookup = HashMap::new();
    for (surface, w1, w2) in ms_pairs {
        let key = (w1.clone(), w2.clone());
        lookup.insert(key, surface.clone());
    }
    return lookup;
}

/// Make a set of all known words that are the solution to a multi surface
pub fn make_word_list(ms_pairs: &Vec<(String, Word, Word)>) -> HashSet<Word> {
    let mut words = HashSet::new();
    for (_, w, _) in ms_pairs {
        words.insert(w.clone());
    }
    return words;
}

/// Make a set of all known words of a given size
pub fn make_word_list_all(size: usize, clues: &Vec<(String, String)>) -> HashSet<Word> {
    let mut words = HashSet::new();
    for (_, solution) in clues {
        let word: Word = solution.chars().collect();
        if word.len() == size {
            words.insert(word);
        }
    }
    return words;
}
