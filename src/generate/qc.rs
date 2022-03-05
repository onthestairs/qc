use serde::{Deserialize, Serialize};
use term_table::row::Row;
use term_table::table_cell::{Alignment, TableCell};
use term_table::{Table, TableStyle};

use super::grid::Grid;

/// A quinian crossword
#[derive(PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QuinianCrossword {
    /// the left hand grid
    pub grid1: Grid,
    /// the right hand grid
    pub grid2: Grid,
    /// the across surfaces
    pub across_surfaces: Vec<String>,
    /// the down surfaces
    pub down_surfaces: Vec<String>,
}

/// print a solution
pub fn print_qc(solution: &QuinianCrossword) {
    let mut table = Table::new();
    table.max_column_width = 50;

    table.style = TableStyle::simple();

    table.add_row(Row::new(vec![
        TableCell::new_with_alignment("Grid1", 1, Alignment::Center),
        TableCell::new_with_alignment("Grid2", 1, Alignment::Center),
        TableCell::new_with_alignment("Across clues", 1, Alignment::Center),
        TableCell::new_with_alignment("Down clues", 1, Alignment::Center),
    ]));

    let grid1 = print_grid(&solution.grid1);
    let grid2 = print_grid(&solution.grid2);
    let across_surfaces = print_surfaces(&solution.across_surfaces);
    let down_surfaces = print_surfaces(&solution.down_surfaces);
    table.add_row(Row::new(vec![
        TableCell::new(grid1),
        TableCell::new(grid2),
        TableCell::new(across_surfaces),
        TableCell::new(down_surfaces),
    ]));

    println!("{}", table.render());
}

fn print_grid(grid: &Grid) -> String {
    let mut table = Table::new();
    table.style = TableStyle::extended();

    for i in 0..grid.len() {
        let cells: Vec<TableCell> = grid[i].iter().map(|c| TableCell::new(c)).collect();
        table.add_row(Row::new(cells));
    }
    return table.render();
}

fn print_surfaces(surfaces: &Vec<String>) -> String {
    let mut surfaces_str = String::new();
    for (i, surface) in surfaces.iter().enumerate() {
        let n = i + 1;
        let surface = format!("{n}. {surface}").to_string();
        surfaces_str += &surface;
        surfaces_str += "\n";
    }
    return surfaces_str;
}
