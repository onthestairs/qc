use std::collections::{HashMap, HashSet};

fn should_include(surface: &String, solution: &String, length: usize) -> bool {
    if surface.len() == 0 || surface.starts_with("See") {
        return false;
    }
    if solution.len() != length {
        return false;
    }
    if solution.contains(" ") {
        return false;
    }
    return true;
}

fn accumulate(clues: Vec<(String, String)>, length: usize) -> HashMap<String, HashSet<String>> {
    let mut map: HashMap<String, HashSet<String>> = HashMap::new();
    for (surface, solution) in clues {
        if !should_include(&surface, &solution, length) {
            continue;
        }
        let solutions = map.entry(surface).or_insert(HashSet::new());
        (*solutions).insert(solution);
    }
    return map;
}

/// hello
pub fn get_multi_surfaces(
    clues: Vec<(String, String)>,
    length: usize,
) -> HashMap<String, HashSet<String>> {
    let surface_solutions = accumulate(clues, length);
    let surface_counts: HashMap<String, usize> = surface_solutions
        .iter()
        .map(|(surface, solutions)| (surface.clone(), solutions.len()))
        .collect();
    let surfaces: Vec<String> = surface_solutions.keys().map(|s| s.clone()).collect();
    let multi_surfaces: HashSet<String> = surfaces
        .into_iter()
        .filter(|surface| *surface_counts.get(surface).unwrap() >= 2)
        .collect();
    let multi_surfaces_with_clues = surface_solutions
        .into_iter()
        .filter(|(surface, _)| multi_surfaces.contains(surface))
        .collect();
    return multi_surfaces_with_clues;
}
