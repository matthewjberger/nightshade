use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MorphWeights {
    pub weights: Vec<f32>,
    pub mesh_name: String,
    #[serde(skip)]
    pub applied_weights: Vec<f32>,
    #[serde(skip)]
    pub morphed_mesh_name: Option<String>,
}

impl MorphWeights {
    pub fn new(weights: Vec<f32>, mesh_name: impl Into<String>) -> Self {
        Self {
            weights,
            mesh_name: mesh_name.into(),
            applied_weights: Vec::new(),
            morphed_mesh_name: None,
        }
    }

    pub fn from_count(count: usize, mesh_name: impl Into<String>) -> Self {
        Self {
            weights: vec![0.0; count],
            mesh_name: mesh_name.into(),
            applied_weights: Vec::new(),
            morphed_mesh_name: None,
        }
    }

    pub fn set_weight(&mut self, index: usize, weight: f32) {
        if index < self.weights.len() {
            self.weights[index] = weight;
        }
    }

    pub fn get_weight(&self, index: usize) -> f32 {
        self.weights.get(index).copied().unwrap_or(0.0)
    }

    pub fn weight_count(&self) -> usize {
        self.weights.len()
    }

    pub fn needs_update(&self) -> bool {
        if self.applied_weights.len() != self.weights.len() {
            return true;
        }
        self.weights
            .iter()
            .zip(self.applied_weights.iter())
            .any(|(current, applied)| (current - applied).abs() > 0.0001)
    }

    pub fn mark_applied(&mut self) {
        self.applied_weights = self.weights.clone();
    }
}
