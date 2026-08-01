use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

pub struct Material;
pub struct Mesh;
pub struct Texture;

#[derive(Serialize, Deserialize)]
pub struct Handle<T> {
    pub index: u32,
    pub generation: u32,
    #[serde(skip)]
    _marker: PhantomData<fn() -> T>,
}

impl<T> Handle<T> {
    pub const INVALID: Self = Self {
        index: u32::MAX,
        generation: 0,
        _marker: PhantomData,
    };

    pub const fn new(index: u32, generation: u32) -> Self {
        Self {
            index,
            generation,
            _marker: PhantomData,
        }
    }

    pub fn is_valid(self) -> bool {
        self.index != u32::MAX
    }
}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Handle<T> {}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}

impl<T> Eq for Handle<T> {}

impl<T> Hash for Handle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.generation.hash(state);
    }
}

impl<T> Default for Handle<T> {
    fn default() -> Self {
        Self::INVALID
    }
}

impl<T> std::fmt::Debug for Handle<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct(std::any::type_name::<Self>())
            .field("index", &self.index)
            .field("generation", &self.generation)
            .finish()
    }
}

pub type MaterialId = Handle<Material>;
pub type MeshId = Handle<Mesh>;
pub type TextureId = Handle<Texture>;

pub const DEFAULT_MATERIAL_ID: MaterialId = MaterialId::new(0, 0);
