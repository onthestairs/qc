use std::collections::HashMap;

use qc::munge::get_multi_surfaces;
use qc::store::csv::get_clues;

fn main() {
    let clues = get_clues().unwrap();
    let multi_surfaces = get_multi_surfaces(clues, 4);
    println!("There are {} multi surfaces", multi_surfaces.len());
    let surface_counts: HashMap<String, usize> = multi_surfaces
        .iter()
        .map(|(surface, solutions)| (surface.clone(), solutions.len()))
        .collect();
    let mut surfaces: Vec<String> = multi_surfaces.keys().map(|s| s.clone()).collect();
    surfaces.sort_by(|a, b| {
        let a_count = surface_counts.get(a).unwrap();
        let b_count = surface_counts.get(b).unwrap();
        return a_count.cmp(b_count);
    });
    let top_surfaces = surfaces.iter().rev().take(50);
    for surface in top_surfaces {
        let surface_solutions = multi_surfaces.get(surface).unwrap();
        let solutions_str = surface_solutions
            .iter()
            .map(|s| s.clone())
            .collect::<Vec<String>>()
            .join(", ");
        println!("{surface}: {solutions_str}");
    }
}
