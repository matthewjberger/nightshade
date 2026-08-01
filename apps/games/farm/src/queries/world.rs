use crate::queries::terrain::is_starting_plot;
use crate::resources::Terrain;

pub fn starting_plots(terrain: &Terrain) -> Vec<(u32, u32)> {
    (0..terrain.rows)
        .flat_map(|row| (0..terrain.columns).map(move |column| (column, row)))
        .filter(|(column, row)| is_starting_plot(*column, *row))
        .collect()
}
