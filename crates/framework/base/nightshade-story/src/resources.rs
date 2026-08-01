use crate::data::story::Place;

#[derive(Clone, Debug, Default)]
pub struct Play {
    pub at: usize,
    pub places: Vec<Place>,
    pub values: Vec<(String, i32)>,
    pub told: Vec<String>,
    pub ended: bool,
}
