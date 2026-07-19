//! Generational resource handles and their type tags.

use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// Type tag marking a [`Handle`] as referring to a material.
pub struct Material;
/// Type tag marking a [`Handle`] as referring to a mesh.
pub struct Mesh;
/// Type tag marking a [`Handle`] as referring to a texture.
pub struct Texture;

/// Generational handle to a `T` resource, pairing a slot index with a generation.
#[derive(Serialize, Deserialize)]
pub struct Handle<T> {
    /// Slot index into the owning registry.
    pub index: u32,
    /// Generation stamp guarding against stale reuse of the slot.
    pub generation: u32,
    #[serde(skip)]
    _marker: PhantomData<fn() -> T>,
}

impl<T> Handle<T> {
    /// Sentinel handle that refers to no resource.
    pub const INVALID: Self = Self {
        index: u32::MAX,
        generation: 0,
        _marker: PhantomData,
    };

    /// Builds a handle from a slot index and generation.
    pub const fn new(index: u32, generation: u32) -> Self {
        Self {
            index,
            generation,
            _marker: PhantomData,
        }
    }

    /// Returns `true` when the index is not the invalid sentinel.
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

/// Handle identifying a material.
pub type MaterialId = Handle<Material>;
/// Handle identifying a mesh.
pub type MeshId = Handle<Mesh>;
/// Handle identifying a texture.
pub type TextureId = Handle<Texture>;
