use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::{
    ArchetypeEdges, ArchetypeRouting, Entity, EntityAllocator, EntityLocation, EntityLocations,
    EventChannel, PlannedTable, STRUCTURAL_LOG_CAPACITY, SparseTagSet, StructuralChange,
    StructuralChangeKind, archetype_cached_tables, archetype_planned_tables,
    archetype_read_cached_tables, archetype_read_planned_tables, archetype_register_table,
    tick_is_newer,
};

static NEXT_REGISTRY_ID: AtomicU32 = AtomicU32::new(1);

pub type BoxedAny = Box<dyn Any + Send + Sync>;

pub type StagedComponents = HashMap<(TypeId, Entity), BoxedAny>;

mod bundle;
mod hierarchy;
mod mask;
mod query;
mod registry;
mod resources;
mod stats;
mod storage;
mod world;

pub use bundle::*;
pub use hierarchy::*;
pub use mask::*;
pub use query::*;
pub use registry::*;
pub use resources::*;
pub use stats::*;
pub use storage::*;
pub use world::*;

mod sealed {
    pub trait SealedElement {}
    pub trait SealedQueryTuple {}
    pub trait SealedBundle {}
    pub trait SealedResourceBundle {}
}

#[doc(hidden)]
pub use sealed::SealedBundle;
