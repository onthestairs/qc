//! Munge relevant data

use itertools::Itertools;
use std::collections::{HashMap, HashSet};

/// type alias for a word
pub type Word = Vec<char>;

/// type alias for a clue surface
pub type Clue = String;

/// type alias for two answers with the same clue.
pub type MultiSurface = (Clue, Word, Word);

/// find all surfaces with many solutions of length `length`
pub fn get_multi_surfaces(clues: &Vec<(String, Word)>) -> HashMap<String, HashSet<Word>> {
    // make a map from surface to a set of solutions
    let mut surface_solutions: HashMap<String, HashSet<Word>> = HashMap::new();
    for (surface, solution) in clues {
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
pub fn make_ms_pairs(multi_surfaces: &HashMap<String, HashSet<Word>>) -> Vec<MultiSurface> {
    let mut pairs = vec![];
    for (surface, solutions) in multi_surfaces {
        let solutions_vec: Vec<&Word> = solutions.iter().collect();
        for pair in solutions_vec.iter().combinations(2) {
            let w1: Word = pair[0].to_vec();
            let w2: Word = pair[1].to_vec();
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
pub type PairPrefixLookup = HashMap<(Word, Word), Vec<MultiSurface>>;

/// Type for doing a lookup by first and third letters of a pair of answers
pub type PairFirstAndThirdLookup = HashMap<(Word, Word), Vec<MultiSurface>>;

/// Make a lookup from the 2-prefix of each word in a pair, to the
/// possible pairs
pub fn make_pair_prefix_lookup(pairs: &Vec<MultiSurface>) -> PairPrefixLookup {
    let mut lookup: PairPrefixLookup = HashMap::new();
    for (s, w1, w2) in pairs {
        let prefix1 = w1.iter().take(2).map(|c| c.clone()).collect();
        let prefix2 = w2.iter().take(2).map(|c| c.clone()).collect();
        let key_words = lookup.entry((prefix1, prefix2)).or_insert(vec![]);
        (*key_words).push((s.clone(), w1.clone(), w2.clone()));
    }
    return lookup;
}

/// Make a lookup from the 1st and 3rd characters of each word in a pair, to the
/// possible pairs
pub fn make_first_and_third_prefix_lookup(pairs: &Vec<MultiSurface>) -> PairFirstAndThirdLookup {
    let mut lookup: PairPrefixLookup = HashMap::new();
    for (s, w1, w2) in pairs {
        // let prefix1 =  w1.iter().take(2).map(|c| c.clone()).collect();
        // let prefix2 = w2.iter().take(2).map(|c| c.clone()).collect();
        let lookup1 = vec![w1[0], w1[2]];
        let lookup2 = vec![w2[0], w2[2]];
        let key_words = lookup.entry((lookup1, lookup2)).or_insert(vec![]);
        (*key_words).push((s.clone(), w1.clone(), w2.clone()));
    }
    return lookup;
}

/// Make a map from a given pair to its surface
pub fn make_pairs_to_surface(
    ms_pairs: &Vec<MultiSurface>,
) -> HashMap<(Word, Word), String> {
    let mut lookup = HashMap::new();
    for (surface, w1, w2) in ms_pairs {
        let key = (w1.clone(), w2.clone());
        lookup.insert(key, surface.clone());
    }
    return lookup;
}

/// Make a set of all known words that are the solution to a multi surface
pub fn make_word_list(ms_pairs: &Vec<MultiSurface>) -> HashSet<Word> {
    let mut words = HashSet::new();
    for (_, w, _) in ms_pairs {
        words.insert(w.clone());
    }
    return words;
}

/// Make a set of all known words of a given size
pub fn make_word_list_all(size: usize, clues: &Vec<(String, Word)>) -> HashSet<Word> {
    let mut words = HashSet::new();
    for (_, solution) in clues {
        if solution.len() == size {
            words.insert(solution.clone());
        }
    }
    return words;
}
