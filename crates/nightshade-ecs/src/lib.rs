//! An archetype-based Entity Component System with runtime component registration.
//!
//! Entities that share a component set are stored together in contiguous,
//! struct-of-arrays storage, so iteration stays cache-coherent and parallelizes
//! across cores on native targets. Components register at runtime through the
//! dynamic world API rather than through generic parameters.
//!
//! # Key Features
//!
//! - **Runtime Registration**: Register components at runtime and derive archetypes from the
//!   component types in a bundle spawn
//! - **Parallel Processing**: Multi-threaded iteration using Rayon (automatically enabled on
//!   non-WASM platforms)
//! - **Sparse Set Tags**: Deterministic, generation-checked markers that don't fragment archetypes
//! - **Command Buffers**: Queue structural changes during iteration
//! - **Change Detection**: Track component modifications for incremental updates
//! - **Events**: Sequence-numbered channels with exactly-once cursor consumption
//! - **Structural Change Log**: Cursor-based log of spawns, despawns, component moves, and tag flips
//! - **Multi-World**: Split components across multiple member worlds so a group can exceed 64
//!   component types
//! - **Snapshots and Deltas** (optional `snapshot` feature): serialize a world and stream
//!   incremental changes
//! - **States** (optional `state` feature): a state-machine layer over the dynamic scheduler
//!
//! # Quick Start
//!
//! ```rust
//! #[derive(Default, Clone, Debug)]
//! struct Position { x: f32, y: f32 }
//!
//! #[derive(Default, Clone, Debug)]
//! struct Velocity { x: f32, y: f32 }
//!
//! nightshade_ecs::dynamic_schema! {
//!     pub fn register_components {
//!         position: Position => POSITION,
//!         velocity: Velocity => VELOCITY,
//!     }
//! }
//!
//! let world = nightshade_ecs::dynamic::DynWorld::from_registry(register_components());
//! assert_eq!(POSITION, 1);
//! assert_eq!(VELOCITY, 2);
//! assert_eq!(world.remaining_bits(), 62);
//! ```

pub use paste;

#[cfg(feature = "dynamic")]
pub mod dynamic;

#[cfg(feature = "dynamic")]
pub mod system_param;

pub mod easing;
#[cfg(feature = "state")]
pub mod state;
pub mod time;

/// Declares a dynamic world's schema in one place: the mask constants (bits
/// assigned in declaration order, which is the registration order and
/// therefore the snapshot schema) and the registration function that builds
/// a [`dynamic::ComponentRegistry`] in that exact order, asserting each
/// key's mask against its constant. Declare every component on every build
/// configuration, and only ever append, so masks stay identical across
/// feature sets and saves stay loadable.
///
/// The leading field name documents intent; only the type and constant are
/// used. Prefix the function with `serde` to register every component with
/// a snapshot codec (requires the `snapshot` feature and serde derives on
/// the components).
///
/// ```rust
/// #[derive(Default, Clone, Debug)]
/// struct Position { x: f32, y: f32 }
///
/// #[derive(Default, Clone, Debug)]
/// struct Velocity { x: f32, y: f32 }
///
/// nightshade_ecs::dynamic_schema! {
///     pub fn register_components {
///         position: Position => POSITION,
///         velocity: Velocity => VELOCITY,
///     }
/// }
///
/// let world = nightshade_ecs::dynamic::DynWorld::from_registry(register_components());
/// assert_eq!(POSITION, 1);
/// assert_eq!(VELOCITY, 2);
/// assert_eq!(world.remaining_bits(), 62);
/// ```
#[cfg(feature = "dynamic")]
#[macro_export]
macro_rules! dynamic_schema {
    (@consts $bit:expr;) => {};
    (@consts $bit:expr; $const:ident $(, $rest:ident)*) => {
        pub const $const: u64 = $bit;
        const _: () = assert!(
            $const != 0,
            "dynamic_schema! supports at most 64 components per world"
        );
        $crate::dynamic_schema!(@consts $bit << 1; $($rest),*);
    };
    (
        $vis:vis fn $register_fn:ident {
            $($field:ident: $ty:ty => $const:ident,)+
        }
    ) => {
        $crate::dynamic_schema!(@consts 1u64; $($const),+);

        $vis fn $register_fn() -> $crate::dynamic::ComponentRegistry {
            let mut registry = $crate::dynamic::ComponentRegistry::new();
            $(
                let key = registry.register::<$ty>();
                assert_eq!(
                    key.mask,
                    $const,
                    "schema declaration order must match registration order"
                );
            )+
            registry
        }
    };
    (
        serde $vis:vis fn $register_fn:ident {
            $($field:ident: $ty:ty => $const:ident,)+
        }
    ) => {
        $crate::dynamic_schema!(@consts 1u64; $($const),+);

        $vis fn $register_fn() -> $crate::dynamic::ComponentRegistry {
            let mut registry = $crate::dynamic::ComponentRegistry::new();
            $(
                let key = registry.register_serde::<$ty>();
                assert_eq!(
                    key.mask,
                    $const,
                    "schema declaration order must match registration order"
                );
            )+
            registry
        }
    };
}

/// Marks one or more types as a [`dynamic::Component`] so they may take part
/// in a bundle. Mark each type exactly once. The types must be local to the
/// calling crate (the orphan rule); wrap a foreign type such as a primitive in
/// a newtype to mark it.
///
/// ```rust
/// #[derive(Default, Clone)]
/// struct Position { x: f32 }
///
/// nightshade_ecs::impl_component!(Position);
/// ```
#[cfg(feature = "dynamic")]
#[macro_export]
macro_rules! impl_component {
    ($($ty:ty),+ $(,)?) => {
        $(impl $crate::dynamic::Component for $ty {})+
    };
}

/// Declares a named bundle struct plus a consuming builder. Each field is
/// itself a bundle, a single [`dynamic::Component`] or a nested bundle struct,
/// so a bundle flattens its fields into one component set on spawn and
/// composes with other bundles and tuples. Anywhere a bundle is accepted takes
/// the struct: `spawn`, `spawn_bundles`, `queue_spawn`, and
/// [`dynamic::DynEcs::spawn_with`]. The struct, its attributes, and each
/// field's visibility pass through verbatim. The macro provides `Default`
/// field by field (do not derive it yourself); `Name::builder()` returns that
/// default and each field name is a consuming setter returning `Self`, so the
/// value spawns directly. No `Clone` on the struct is needed; batch spawns and
/// takes work when the fields are `Clone` and `Default`, which components
/// already are. Repeating a component type across the flattened set panics on
/// spawn.
///
/// ```rust
/// use nightshade_ecs::dynamic::DynWorld;
///
/// #[derive(Default, Clone, PartialEq)]
/// struct Position { x: f32 }
/// #[derive(Default, Clone, PartialEq)]
/// struct Velocity { x: f32 }
/// #[derive(Default, Clone, PartialEq)]
/// struct Health { value: f32 }
///
/// nightshade_ecs::impl_component!(Position, Velocity, Health);
///
/// nightshade_ecs::bundle! {
///     struct Motion {
///         position: Position,
///         velocity: Velocity,
///     }
/// }
///
/// nightshade_ecs::bundle! {
///     pub struct Actor {
///         pub motion: Motion,
///         pub health: Health,
///     }
/// }
///
/// let mut world = DynWorld::new();
/// let entity = world.spawn(
///     Actor::builder()
///         .motion(Motion {
///             position: Position { x: 1.0 },
///             velocity: Velocity { x: 2.0 },
///         })
///         .health(Health { value: 100.0 }),
/// );
/// assert_eq!(world.get::<Position>(entity).unwrap().x, 1.0);
/// assert_eq!(world.get::<Velocity>(entity).unwrap().x, 2.0);
/// assert_eq!(world.get::<Health>(entity).unwrap().value, 100.0);
/// ```
#[cfg(feature = "dynamic")]
#[macro_export]
macro_rules! bundle {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $($fvis:vis $field:ident : $ty:ty),+ $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis struct $name {
            $($fvis $field: $ty),+
        }

        impl ::core::default::Default for $name {
            fn default() -> Self {
                Self {
                    $($field: ::core::default::Default::default()),+
                }
            }
        }

        impl $name {
            $vis fn builder() -> Self {
                <Self as ::core::default::Default>::default()
            }

            $(
                $vis fn $field(mut self, value: $ty) -> Self {
                    self.$field = value;
                    self
                }
            )+
        }

        impl $crate::dynamic::SealedBundle for $name {}

        impl $crate::dynamic::Bundle for $name {
            fn component_mask(world: &mut $crate::dynamic::DynWorld) -> u64 {
                let mut mask = 0u64;
                $(
                    let element_mask =
                        <$ty as $crate::dynamic::Bundle>::component_mask(world);
                    assert_eq!(
                        mask & element_mask,
                        0,
                        "bundle! structs must not repeat a component type"
                    );
                    mask |= element_mask;
                )+
                mask
            }

            fn lookup_mask(world: &$crate::dynamic::DynWorld) -> u64 {
                let mut mask = 0u64;
                $(mask |= <$ty as $crate::dynamic::Bundle>::lookup_mask(world);)+
                mask
            }

            fn write(self, world: &mut $crate::dynamic::DynWorld, entity: $crate::Entity) {
                $($crate::dynamic::Bundle::write(self.$field, world, entity);)+
            }

            fn take(world: &mut $crate::dynamic::DynWorld, entity: $crate::Entity) -> Self {
                Self {
                    $($field: <$ty as $crate::dynamic::Bundle>::take(world, entity),)+
                }
            }

            fn write_group(self, ecs: &mut $crate::dynamic::DynEcs, entity: $crate::Entity) {
                $($crate::dynamic::Bundle::write_group(self.$field, ecs, entity);)+
            }

            fn remove_group(ecs: &mut $crate::dynamic::DynEcs, entity: $crate::Entity) -> bool {
                let mut removed = false;
                $(
                    removed |=
                        <$ty as $crate::dynamic::Bundle>::remove_group(ecs, entity);
                )+
                removed
            }

            fn present_group(ecs: &$crate::dynamic::DynEcs, entity: $crate::Entity) -> bool {
                true $(&& <$ty as $crate::dynamic::Bundle>::present_group(ecs, entity))+
            }

            fn take_group(ecs: &mut $crate::dynamic::DynEcs, entity: $crate::Entity) -> Self {
                Self {
                    $($field: <$ty as $crate::dynamic::Bundle>::take_group(ecs, entity),)+
                }
            }
        }

        impl $crate::dynamic::CloneBundle for $name {
            fn spawn_extend(
                &self,
                world: &mut $crate::dynamic::DynWorld,
                table_index: usize,
                count: usize,
            ) {
                $(
                    $crate::dynamic::CloneBundle::spawn_extend(
                        &self.$field,
                        world,
                        table_index,
                        count,
                    );
                )+
            }
        }
    };
}

/// Declares a [`dynamic::DynEcs`] group's member worlds in one place: the
/// index constants in declaration order and the build function that adds
/// each member's registry at its asserted index, replacing the hand-written
/// const-add-assert dance. Pair each member with a registration function,
/// typically from [`dynamic_schema!`]. Apps extending a built group add
/// their own member with [`dynamic::DynEcs::add_world_at`].
///
/// ```rust
/// #[derive(Default, Clone)]
/// struct Value(u32);
///
/// nightshade_ecs::dynamic_schema! {
///     pub fn register_main {
///         value: Value => VALUE,
///     }
/// }
///
/// nightshade_ecs::dynamic_worlds! {
///     pub fn build_ecs {
///         MAIN => register_main,
///     }
/// }
///
/// let ecs = build_ecs();
/// assert_eq!(MAIN, 0);
/// assert_eq!(ecs.worlds.len(), 1);
/// ```
#[cfg(feature = "dynamic")]
#[macro_export]
macro_rules! dynamic_worlds {
    (@consts $index:expr;) => {};
    (@consts $index:expr; $const:ident $(, $rest:ident)*) => {
        pub const $const: usize = $index;
        $crate::dynamic_worlds!(@consts $index + 1; $($rest),*);
    };
    (
        $vis:vis fn $build_fn:ident {
            $($const:ident => $register_fn:ident,)+
        }
    ) => {
        $crate::dynamic_worlds!(@consts 0usize; $($const),+);

        $vis fn $build_fn() -> $crate::dynamic::DynEcs {
            let mut ecs = $crate::dynamic::DynEcs::new();
            $(
                ecs.add_world_at($const, $register_fn());
            )+
            ecs
        }
    };
}

/// Generates the macro-world accessor ergonomics over the keyed tier: a
/// keys struct holding one [`dynamic::ComponentKey`] or
/// [`dynamic::TagKey`] per entry, a `resolve` constructor registering them
/// in declaration order, and named methods on your wrapper type â€”
/// `get_<name>` / `get_<name>_mut` / `set_<name>` / `remove_<name>` /
/// `has_<name>` per component, `add_<name>` / `remove_<name>` /
/// `has_<name>` / `query_<name>` per tag. The wrapper must expose the named
/// world and keys fields. Accessors run at keyed speed and stamp change
/// ticks exactly like the generated macro world's. Component and tag names
/// share one method namespace (`remove_` and `has_` exist for both), so an
/// identifier may appear in only one of the two blocks; reusing one is a
/// duplicate-method compile error.
///
/// ```rust
/// use nightshade_ecs::dynamic::DynWorld;
///
/// #[derive(Default, Clone, Debug)]
/// struct Position { x: f32 }
///
/// nightshade_ecs::impl_component!(Position);
///
/// struct Boss;
///
/// struct Game {
///     world: DynWorld,
///     keys: GameKeys,
/// }
///
/// nightshade_ecs::dynamic_accessors! {
///     pub struct GameKeys for Game { world, keys }
///     components {
///         position: Position,
///     }
///     tags {
///         boss: Boss,
///     }
/// }
///
/// let mut world = DynWorld::new();
/// let keys = GameKeys::resolve(&mut world);
/// let mut game = Game { world, keys };
///
/// let entity = game.world.spawn((Position { x: 1.0 },));
/// game.set_position(entity, Position { x: 2.0 });
/// game.add_boss(entity);
/// assert_eq!(game.get_position(entity).unwrap().x, 2.0);
/// assert!(game.has_boss(entity));
/// assert_eq!(game.query_boss().count(), 1);
/// ```
#[cfg(feature = "dynamic")]
#[macro_export]
macro_rules! dynamic_accessors {
    (
        $vis:vis struct $keys:ident for $wrapper:ident { $world_field:ident, $keys_field:ident }
        components {
            $($component:ident: $component_type:ty,)*
        }
        tags {
            $($tag:ident: $tag_type:ty,)*
        }
    ) => {
        $vis struct $keys {
            $(pub $component: $crate::dynamic::ComponentKey<$component_type>,)*
            $(pub $tag: $crate::dynamic::TagKey,)*
        }

        impl $keys {
            $vis fn resolve(world: &mut $crate::dynamic::DynWorld) -> Self {
                Self {
                    $($component: world.register::<$component_type>(),)*
                    $($tag: world.tag_key::<$tag_type>(),)*
                }
            }
        }

        $crate::paste::paste! {
            impl $wrapper {
                $(
                    $vis fn [<get_ $component>](&self, entity: $crate::Entity) -> Option<&$component_type> {
                        self.$world_field.get_keyed(self.$keys_field.$component, entity)
                    }

                    $vis fn [<get_ $component _mut>](&mut self, entity: $crate::Entity) -> Option<&mut $component_type> {
                        self.$world_field.get_mut_keyed(self.$keys_field.$component, entity)
                    }

                    $vis fn [<set_ $component>](&mut self, entity: $crate::Entity, value: $component_type) {
                        self.$world_field.set_keyed(self.$keys_field.$component, entity, value);
                    }

                    $vis fn [<remove_ $component>](&mut self, entity: $crate::Entity) -> bool {
                        self.$world_field.remove_components(entity, self.$keys_field.$component.mask)
                    }

                    $vis fn [<has_ $component>](&self, entity: $crate::Entity) -> bool {
                        self.$world_field.entity_has_components(entity, self.$keys_field.$component.mask)
                    }
                )*

                $(
                    $vis fn [<add_ $tag>](&mut self, entity: $crate::Entity) {
                        self.$world_field.add_tag(self.$keys_field.$tag, entity);
                    }

                    $vis fn [<remove_ $tag>](&mut self, entity: $crate::Entity) -> bool {
                        self.$world_field.remove_tag(self.$keys_field.$tag, entity)
                    }

                    $vis fn [<has_ $tag>](&self, entity: $crate::Entity) -> bool {
                        self.$world_field.has_tag(self.$keys_field.$tag, entity)
                    }

                    $vis fn [<query_ $tag>](&self) -> impl Iterator<Item = $crate::Entity> + '_ {
                        self.$world_field.query_tag(self.$keys_field.$tag)
                    }
                )*
            }
        }
    };
}

#[cfg(not(target_family = "wasm"))]
pub use rayon;

#[derive(Default, Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Entity {
    pub id: u32,
    pub generation: u32,
}

impl std::fmt::Display for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { id, generation } = self;
        write!(f, "Id: {id} - Generation: {generation}")
    }
}

/// Liveness record for one entity id: the generation currently associated
/// with the id and whether that handle is live.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EntitySlot {
    pub generation: u32,
    pub alive: bool,
}

/// Allocates generational entity handles and tracks which handles are live.
///
/// Liveness is authoritative here: `deallocate` refuses stale or already-freed
/// handles, so an id can never enter the free list twice and two live entities
/// can never share an id and generation.
#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EntityAllocator {
    pub next_id: u32,
    pub free_ids: Vec<(u32, u32)>,
    pub slots: Vec<EntitySlot>,
}

impl EntityAllocator {
    /// Forces a handle live at its exact id and generation, the primitive a
    /// replica uses when applying a replicated spawn. Extends the slot
    /// table as needed, removes the id from the free list, and keeps
    /// `next_id` ahead of it.
    pub fn revive(&mut self, entity: Entity) {
        let index = entity.id as usize;
        if self.slots.len() <= index {
            self.slots.resize(
                index + 1,
                EntitySlot {
                    generation: 0,
                    alive: false,
                },
            );
        }
        self.slots[index] = EntitySlot {
            generation: entity.generation,
            alive: true,
        };
        self.free_ids.retain(|&(id, _)| id != entity.id);
        if self.next_id <= entity.id {
            self.next_id = entity.id + 1;
        }
    }

    #[inline]
    pub fn allocate(&mut self) -> Entity {
        let entity = if let Some((id, next_generation)) = self.free_ids.pop() {
            Entity {
                id,
                generation: next_generation,
            }
        } else {
            let id = self.next_id;
            self.next_id += 1;
            Entity { id, generation: 0 }
        };
        let index = entity.id as usize;
        if index >= self.slots.len() {
            self.slots.resize(index + 1, EntitySlot::default());
        }
        self.slots[index] = EntitySlot {
            generation: entity.generation,
            alive: true,
        };
        entity
    }

    /// Allocates `count` handles into `entities`, recycling freed ids first.
    /// Fresh ids are contiguous, so their liveness slots are written with one
    /// bulk fill instead of a store per entity.
    pub fn allocate_batch(&mut self, count: usize, entities: &mut Vec<Entity>) {
        entities.reserve(count);
        let recycled = count.min(self.free_ids.len());
        for _ in 0..recycled {
            let Some((id, generation)) = self.free_ids.pop() else {
                break;
            };
            let index = id as usize;
            if index >= self.slots.len() {
                self.slots.resize(index + 1, EntitySlot::default());
            }
            self.slots[index] = EntitySlot {
                generation,
                alive: true,
            };
            entities.push(Entity { id, generation });
        }

        let fresh = count - recycled;
        if fresh > 0 {
            let start = self.next_id;
            self.next_id += fresh as u32;
            let start_index = start as usize;
            let end_index = start_index + fresh;
            if end_index > self.slots.len() {
                self.slots.resize(end_index, EntitySlot::default());
            }
            self.slots[start_index..end_index].fill(EntitySlot {
                generation: 0,
                alive: true,
            });
            for id in start..self.next_id {
                entities.push(Entity { id, generation: 0 });
            }
        }
    }

    /// Frees the handle if it is currently live. Returns false for stale
    /// generations and double frees, leaving the allocator untouched.
    #[inline]
    pub fn deallocate(&mut self, entity: Entity) -> bool {
        if !self.is_alive(entity) {
            return false;
        }
        self.slots[entity.id as usize].alive = false;
        self.free_ids
            .push((entity.id, entity.generation.wrapping_add(1)));
        true
    }

    #[inline]
    pub fn is_alive(&self, entity: Entity) -> bool {
        self.slots
            .get(entity.id as usize)
            .is_some_and(|slot| slot.alive && slot.generation == entity.generation)
    }
}

#[derive(Copy, Clone, Default)]
pub(crate) struct EntityLocation {
    pub generation: u32,
    pub table_index: u32,
    pub array_index: u32,
    pub allocated: bool,
}

#[derive(Default)]
pub(crate) struct EntityLocations {
    pub locations: Vec<EntityLocation>,
}

impl EntityLocations {
    pub fn get(&self, id: u32) -> Option<&EntityLocation> {
        self.locations.get(id as usize)
    }

    pub fn get_mut(&mut self, id: u32) -> Option<&mut EntityLocation> {
        self.locations.get_mut(id as usize)
    }

    pub fn ensure_slot(&mut self, id: u32, generation: u32) {
        let id_usize = id as usize;
        if id_usize >= self.locations.len() {
            self.locations.resize(
                (self.locations.len() * 2).max(64).max(id_usize + 1),
                EntityLocation::default(),
            );
        }
        self.locations[id_usize].generation = generation;
    }

    pub fn insert(&mut self, id: u32, location: EntityLocation) {
        let id_usize = id as usize;
        if id_usize >= self.locations.len() {
            self.locations.resize(
                (self.locations.len() * 2).max(64).max(id_usize + 1),
                EntityLocation::default(),
            );
        }
        self.locations[id_usize] = location;
    }

    pub fn mark_deallocated(&mut self, id: u32) {
        if let Some(loc) = self.locations.get_mut(id as usize) {
            loc.allocated = false;
        }
    }
}

/// Returns true if `tick` was stamped after `since_tick`, treating ticks as a
/// wrapping sequence so detection keeps working after `u32` overflow.
#[inline]
pub const fn tick_is_newer(tick: u32, since_tick: u32) -> bool {
    tick.wrapping_sub(since_tick) as i32 > 0
}

const SPARSE_TAG_ABSENT: u32 = u32::MAX;

/// A sparse set of entities backing one tag.
///
/// Dense storage gives contiguous, deterministic iteration; the sparse index
/// gives O(1) insert, remove, and contains without hashing. Membership is
/// generation-checked, so a stale handle never matches a reused id.
///
/// # Examples
///
/// ```
/// use nightshade_ecs::{Entity, SparseTagSet};
///
/// let mut set = SparseTagSet::default();
/// let entity = Entity { id: 3, generation: 0 };
///
/// assert!(set.insert(entity));
/// assert!(set.contains(entity));
/// assert_eq!(set.iter().collect::<Vec<_>>(), vec![entity]);
///
/// let stale = Entity { id: 3, generation: 1 };
/// assert!(!set.contains(stale));
///
/// assert!(set.remove(entity));
/// assert!(set.is_empty());
/// ```
#[derive(Default, Clone, Debug)]
pub struct SparseTagSet {
    pub dense: Vec<Entity>,
    pub sparse: Vec<u32>,
}

impl SparseTagSet {
    /// Adds the entity, replacing any stale entry for the same id. Returns
    /// false if this exact handle was already present.
    #[inline]
    pub fn insert(&mut self, entity: Entity) -> bool {
        let index = entity.id as usize;
        if index >= self.sparse.len() {
            self.sparse.resize(index + 1, SPARSE_TAG_ABSENT);
        }
        let slot = self.sparse[index];
        if slot != SPARSE_TAG_ABSENT {
            let existing = &mut self.dense[slot as usize];
            if *existing == entity {
                return false;
            }
            *existing = entity;
            return true;
        }
        self.sparse[index] = self.dense.len() as u32;
        self.dense.push(entity);
        true
    }

    #[inline]
    pub fn remove(&mut self, entity: Entity) -> bool {
        let index = entity.id as usize;
        let Some(&slot) = self.sparse.get(index) else {
            return false;
        };
        if slot == SPARSE_TAG_ABSENT || self.dense[slot as usize] != entity {
            return false;
        }
        self.dense.swap_remove(slot as usize);
        self.sparse[index] = SPARSE_TAG_ABSENT;
        if (slot as usize) < self.dense.len() {
            let moved = self.dense[slot as usize];
            self.sparse[moved.id as usize] = slot;
        }
        true
    }

    #[inline]
    pub fn contains(&self, entity: Entity) -> bool {
        self.sparse
            .get(entity.id as usize)
            .is_some_and(|&slot| slot != SPARSE_TAG_ABSENT && self.dense[slot as usize] == entity)
    }

    pub fn iter(&self) -> impl Iterator<Item = Entity> + '_ {
        self.dense.iter().copied()
    }

    pub fn len(&self) -> usize {
        self.dense.len()
    }

    pub fn is_empty(&self) -> bool {
        self.dense.is_empty()
    }

    pub fn clear(&mut self) {
        self.dense.clear();
        self.sparse.fill(SPARSE_TAG_ABSENT);
    }
}

/// Archetype graph edges for one table: which table an entity lands in when a
/// single component bit is added or removed, plus memoized targets for
/// multi-bit changes. Shared by the macro-generated worlds and the dynamic
/// world, since none of it depends on component types.
#[derive(Clone, Default)]
pub(crate) struct ArchetypeEdges {
    pub add_edges: Vec<Option<usize>>,
    pub remove_edges: Vec<Option<usize>>,
    pub multi_add_cache: std::collections::HashMap<u64, usize>,
    pub multi_remove_cache: std::collections::HashMap<u64, usize>,
}

impl ArchetypeEdges {
    pub fn new(component_count: usize) -> Self {
        Self {
            add_edges: vec![None; component_count],
            remove_edges: vec![None; component_count],
            multi_add_cache: std::collections::HashMap::default(),
            multi_remove_cache: std::collections::HashMap::default(),
        }
    }
}

/// Registers a newly pushed table with the archetype routing structures:
/// inserts the mask lookup, appends the table's edge record, extends every
/// query-cache entry the new table satisfies, and wires single-component
/// edges from existing tables toward the new one. `table_masks` must iterate
/// every table including the new one, in index order.
pub(crate) struct ArchetypeRouting<'world, S = std::collections::hash_map::RandomState> {
    pub table_lookup: &'world mut std::collections::HashMap<u64, usize>,
    pub table_edges: &'world mut Vec<ArchetypeEdges>,
    pub query_cache: &'world mut std::collections::HashMap<u64, Vec<usize>, S>,
}

pub(crate) fn archetype_register_table<M, S>(
    routing: ArchetypeRouting<'_, S>,
    component_count: usize,
    mask: u64,
    table_index: usize,
    table_masks: M,
    component_bits: impl Iterator<Item = (u64, usize)>,
) where
    M: Iterator<Item = u64> + Clone,
    S: std::hash::BuildHasher,
{
    routing
        .table_edges
        .push(ArchetypeEdges::new(component_count));
    routing.table_lookup.insert(mask, table_index);

    for (query_mask, cached_tables) in routing.query_cache.iter_mut() {
        if mask & *query_mask == *query_mask {
            cached_tables.push(table_index);
        }
    }

    for (component_mask, component_index) in component_bits {
        for (index, existing_mask) in table_masks.clone().enumerate() {
            if existing_mask | component_mask == mask {
                routing.table_edges[index].add_edges[component_index] = Some(table_index);
            }
            if existing_mask & !component_mask == mask {
                routing.table_edges[index].remove_edges[component_index] = Some(table_index);
            }
        }
    }
}

/// Returns the memoized list of table indices whose masks contain `mask`,
/// computing and caching it on first use. Taking the cache and the table
/// masks as separate parameters keeps the borrows disjoint, so callers can
/// mutate tables while holding the returned slice.
pub(crate) fn archetype_cached_tables<M, S>(
    query_cache: &mut std::collections::HashMap<u64, Vec<usize>, S>,
    table_masks: M,
    mask: u64,
) -> &[usize]
where
    M: Iterator<Item = u64>,
    S: std::hash::BuildHasher,
{
    query_cache.entry(mask).or_insert_with(|| {
        table_masks
            .enumerate()
            .filter(|(_, table_mask)| table_mask & mask == mask)
            .map(|(table_index, _)| table_index)
            .collect()
    })
}

/// Backstop for worlds whose structural log is never consumed. When the log
/// reaches this length it is cleared wholesale, so a world with no consumer
/// stays bounded instead of leaking. Consumers that drain every frame never
/// come near it.
pub const STRUCTURAL_LOG_CAPACITY: usize = 262_144;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StructuralChangeKind {
    Spawned,
    Despawned,
    ComponentsAdded,
    ComponentsRemoved,
    TagsAdded,
    TagsRemoved,
}

/// One structural mutation recorded by the world: a spawn, a despawn, or a
/// component add or remove. `mask` holds the components involved: the full
/// mask for spawns and despawns, the delta for adds and removes. Consumers
/// track their own `sequence` cursor via `structural_changes_since` and the
/// owner trims consumed entries with `trim_structural_log`.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StructuralChange {
    pub sequence: u64,
    pub entity: Entity,
    pub kind: StructuralChangeKind,
    pub mask: u64,
}

/// Backstop for event channels whose events are never consumed or expired.
/// When the buffer reaches this length the oldest events are dropped, so a
/// channel with no consumer stays bounded instead of leaking.
pub const EVENT_CHANNEL_CAPACITY: usize = 262_144;

/// A sequence-numbered event channel for inter-system communication.
///
/// Events live in one flat `Vec` and every event gets a monotonically
/// increasing sequence number, the same cursor scheme the structural log
/// uses. Two consumption styles are supported:
///
/// - **Frame-scoped**: [`read()`](EventChannel::read) sees every buffered
///   event, and [`update()`](EventChannel::update) once per frame expires
///   events after they have been visible for two frames, matching the old
///   double-buffer lifetime.
/// - **Cursor-based, exactly-once**: each consumer records
///   [`sequence()`](EventChannel::sequence) after reading
///   [`events_since(cursor)`](EventChannel::events_since). Multiple consumers
///   each see every event exactly once, and a consumer that skips a frame
///   catches up instead of double-processing or missing events.
///
/// # Examples
///
/// ```
/// use nightshade_ecs::EventChannel;
///
/// #[derive(Debug, Clone)]
/// struct DamageEvent { amount: i32 }
///
/// let mut channel = EventChannel::new();
///
/// channel.send(DamageEvent { amount: 10 });
/// assert_eq!(channel.len(), 1);
///
/// let mut cursor = 0;
/// let seen = channel.events_since(cursor);
/// assert_eq!(seen.len(), 1);
/// cursor = channel.sequence();
///
/// assert!(channel.events_since(cursor).is_empty(), "cursor consumers see each event once");
///
/// channel.update();
/// assert_eq!(channel.len(), 1, "event persists after first update");
///
/// channel.update();
/// assert_eq!(channel.len(), 0, "event expired after second update");
/// ```
#[derive(Clone)]
pub struct EventChannel<T> {
    pub events: Vec<T>,
    pub base_sequence: u64,
    pub previous_update_sequence: u64,
}

impl<T> Default for EventChannel<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> EventChannel<T> {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            base_sequence: 0,
            previous_update_sequence: 0,
        }
    }

    /// Sends an event. It stays readable until it expires two `update()`
    /// calls later or a cursor consumer trims past it.
    #[inline]
    pub fn send(&mut self, event: T) {
        if self.events.len() >= EVENT_CHANNEL_CAPACITY {
            self.drop_oldest_half();
        }
        self.events.push(event);
    }

    #[cold]
    fn drop_oldest_half(&mut self) {
        let half = self.events.len() / 2;
        self.events.drain(..half);
        self.base_sequence += half as u64;
    }

    /// The sequence number of the most recently sent event. Record this as
    /// your cursor after consuming `events_since`.
    #[inline]
    pub fn sequence(&self) -> u64 {
        self.base_sequence + self.events.len() as u64
    }

    /// All buffered events sent after `cursor`, oldest first. A cursor older
    /// than the buffer yields everything still buffered.
    #[inline]
    pub fn events_since(&self, cursor: u64) -> &[T] {
        let start = cursor
            .saturating_sub(self.base_sequence)
            .min(self.events.len() as u64) as usize;
        &self.events[start..]
    }

    /// The exactly-once read: yields every event sent after `cursor` and
    /// advances the cursor past them, so calling this every frame delivers
    /// each event to this consumer exactly once. Each consumer owns one
    /// `u64` cursor; the buffer itself is untouched, so other consumers and
    /// the two-frame expiry are unaffected. This is the spelling to reach
    /// for by default; `read()` re-reads the whole buffer every call.
    #[inline]
    pub fn consume(&self, cursor: &mut u64) -> &[T] {
        let events = self.events_since(*cursor);
        *cursor = self.sequence();
        events
    }

    /// Returns an iterator over every buffered event, oldest first.
    pub fn read(&self) -> impl Iterator<Item = &T> {
        self.events.iter()
    }

    /// The events settled at the last [`update`](Self::update): the previous
    /// frame's complete set, frozen and broadcast to every reader without a
    /// cursor. Events sent since that `update` (this frame's sends) are
    /// excluded, so the slice does not change as more events arrive during the
    /// frame. A reader that calls this once per frame therefore sees each
    /// frame's events exactly once, one frame after they were sent, regardless
    /// of where in the frame it reads and of interleaving with senders. This is
    /// the broadcast, single-frame counterpart to the cursor-based
    /// [`consume`](Self::consume): use it when every reader must observe the
    /// same frozen set for a frame, as a deferred event pipeline does.
    #[inline]
    pub fn read_frame(&self) -> &[T] {
        let end = self
            .previous_update_sequence
            .saturating_sub(self.base_sequence)
            .min(self.events.len() as u64) as usize;
        &self.events[..end]
    }

    /// Returns a reference to the oldest buffered event, if any.
    pub fn peek(&self) -> Option<&T> {
        self.events.first()
    }

    /// Drops all events up to and including `up_to_sequence`. Call with the
    /// minimum cursor across consumers to reclaim memory early.
    pub fn trim(&mut self, up_to_sequence: u64) {
        let drop_count = up_to_sequence
            .saturating_sub(self.base_sequence)
            .min(self.events.len() as u64) as usize;
        self.events.drain(..drop_count);
        self.base_sequence += drop_count as u64;
    }

    /// Expires events that have now been visible for two frames. Call once
    /// per frame; `step()` on a generated world does this for every channel.
    pub fn update(&mut self) {
        let expire = self.previous_update_sequence;
        self.trim(expire);
        self.previous_update_sequence = self.sequence();
    }

    /// Immediately drops every buffered event, advancing past them.
    pub fn clear(&mut self) {
        let sequence = self.sequence();
        self.trim(sequence);
    }

    /// Returns the number of buffered events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Returns `true` if no events are buffered.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

struct ScheduleEntry<W> {
    name: &'static str,
    system: Box<dyn FnMut(&mut W) + Send>,
}

pub struct Schedule<W> {
    entries: Vec<ScheduleEntry<W>>,
}

impl<W> Schedule<W> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn push<F>(&mut self, name: &'static str, system: F) -> &mut Self
    where
        F: FnMut(&mut W) + Send + 'static,
    {
        self.assert_unique(name);
        self.entries.push(ScheduleEntry {
            name,
            system: Box::new(system),
        });
        self
    }

    pub fn push_readonly<F>(&mut self, name: &'static str, mut system: F) -> &mut Self
    where
        F: FnMut(&W) + Send + 'static,
    {
        self.assert_unique(name);
        self.entries.push(ScheduleEntry {
            name,
            system: Box::new(move |world: &mut W| {
                system(&*world);
            }),
        });
        self
    }

    /// Adds a system that runs only when the condition holds. The condition
    /// reads the world at each pass; a false skips the system for that pass
    /// without removing it from the schedule.
    pub fn push_if<C, F>(&mut self, name: &'static str, condition: C, mut system: F) -> &mut Self
    where
        C: Fn(&W) -> bool + Send + 'static,
        F: FnMut(&mut W) + Send + 'static,
    {
        self.push(name, move |world: &mut W| {
            if condition(world) {
                system(world);
            }
        })
    }

    pub fn insert_before<F>(&mut self, target: &str, name: &'static str, system: F) -> &mut Self
    where
        F: FnMut(&mut W) + Send + 'static,
    {
        self.assert_unique(name);
        let index = self.index_of_or_panic(target, "insert_before");
        self.entries.insert(
            index,
            ScheduleEntry {
                name,
                system: Box::new(system),
            },
        );
        self
    }

    pub fn insert_after<F>(&mut self, target: &str, name: &'static str, system: F) -> &mut Self
    where
        F: FnMut(&mut W) + Send + 'static,
    {
        self.assert_unique(name);
        let index = self.index_of_or_panic(target, "insert_after");
        self.entries.insert(
            index + 1,
            ScheduleEntry {
                name,
                system: Box::new(system),
            },
        );
        self
    }

    pub fn replace<F>(&mut self, name: &str, system: F) -> &mut Self
    where
        F: FnMut(&mut W) + Send + 'static,
    {
        let index = self
            .index_of(name)
            .unwrap_or_else(|| panic!("Schedule::replace: system \"{name}\" not found"));
        self.entries[index].system = Box::new(system);
        self
    }

    pub fn remove(&mut self, name: &str) -> bool {
        let len_before = self.entries.len();
        self.entries.retain(|entry| entry.name != name);
        self.entries.len() != len_before
    }

    pub fn contains(&self, name: &str) -> bool {
        self.entries.iter().any(|entry| entry.name == name)
    }

    pub fn names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.entries.iter().map(|entry| entry.name)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn run(&mut self, world: &mut W) {
        for entry in &mut self.entries {
            (entry.system)(world);
        }
    }

    fn index_of(&self, name: &str) -> Option<usize> {
        self.entries.iter().position(|entry| entry.name == name)
    }

    fn assert_unique(&self, name: &str) {
        assert!(
            !self.contains(name),
            "Schedule: system \"{name}\" already exists"
        );
    }

    fn index_of_or_panic(&self, target: &str, method: &str) -> usize {
        self.index_of(target)
            .unwrap_or_else(|| panic!("Schedule::{method}: system \"{target}\" not found"))
    }
}

impl<W> Default for Schedule<W> {
    fn default() -> Self {
        Self::new()
    }
}

/// An ordered set of named stages, each its own [`Schedule`], for programs
/// assembled from independent parts: the app declares the stage order once,
/// and each part pushes systems into stages by name, so composition stays
/// deterministic without labels or ordering constraints. Stages run in
/// declaration order; systems within a stage run in push order. Plain data,
/// inspectable through [`stages`](Self::stages).
pub struct Stages<W> {
    pub stages: Vec<(&'static str, Schedule<W>)>,
}

impl<W> Stages<W> {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    /// Appends a stage at the end of the run order. Panics on a duplicate
    /// name, since two parts declaring the same stage is a wiring bug.
    pub fn add_stage(&mut self, name: &'static str) -> &mut Self {
        assert!(
            !self.stages.iter().any(|(existing, _)| *existing == name),
            "stage {name:?} is already declared"
        );
        self.stages.push((name, Schedule::new()));
        self
    }

    /// The stage's schedule, for pushing systems into it
    /// (`stages.stage_mut("simulation").push(...)`). Panics with the
    /// declared stage list when the stage does not exist, so a part pushing
    /// into a missing stage fails at wiring time, not silently.
    pub fn stage_mut(&mut self, name: &str) -> &mut Schedule<W> {
        match self
            .stages
            .iter()
            .position(|(stage_name, _)| *stage_name == name)
        {
            Some(index) => &mut self.stages[index].1,
            None => {
                let declared: Vec<&'static str> = self
                    .stages
                    .iter()
                    .map(|(stage_name, _)| *stage_name)
                    .collect();
                panic!("no stage named {name:?}; declared stages are {declared:?}")
            }
        }
    }

    /// Runs every stage in declaration order.
    pub fn run(&mut self, world: &mut W) {
        for (_name, schedule) in &mut self.stages {
            schedule.run(world);
        }
    }

    /// Runs one stage by name, for loops that drive stages on different
    /// cadences. Panics like [`stage_mut`](Self::stage_mut) when missing.
    pub fn run_stage(&mut self, name: &str, world: &mut W) {
        self.stage_mut(name).run(world);
    }
}

impl<W> Default for Stages<W> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allocator_liveness() {
        let mut allocator = EntityAllocator::default();

        let entity = allocator.allocate();
        assert!(allocator.is_alive(entity));

        assert!(allocator.deallocate(entity));
        assert!(!allocator.is_alive(entity));
        assert!(!allocator.deallocate(entity), "double free must be refused");

        let reused = allocator.allocate();
        assert_eq!(reused.id, entity.id);
        assert_eq!(reused.generation, entity.generation + 1);
        assert!(allocator.is_alive(reused));
        assert!(!allocator.is_alive(entity));

        assert!(
            !allocator.deallocate(entity),
            "stale free must not kill the reused id"
        );
        assert!(allocator.is_alive(reused));

        let other = allocator.allocate();
        assert_ne!((other.id, other.generation), (reused.id, reused.generation));
    }

    #[test]
    fn test_allocator_batch_mixes_recycled_and_fresh() {
        let mut allocator = EntityAllocator::default();

        let first = allocator.allocate();
        let second = allocator.allocate();
        assert!(allocator.deallocate(first));
        assert!(allocator.deallocate(second));

        let mut entities = Vec::new();
        allocator.allocate_batch(4, &mut entities);
        assert_eq!(entities.len(), 4);

        let recycled_count = entities
            .iter()
            .filter(|entity| entity.generation > 0)
            .count();
        assert_eq!(recycled_count, 2, "both freed ids must be recycled first");

        for &entity in &entities {
            assert!(allocator.is_alive(entity));
        }
        assert!(!allocator.is_alive(first));
        assert!(!allocator.is_alive(second));

        let mut seen: Vec<_> = entities.iter().map(|e| (e.id, e.generation)).collect();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), 4, "batch handles must be distinct");
    }

    #[test]
    fn test_tick_is_newer() {
        assert!(tick_is_newer(1, 0));
        assert!(!tick_is_newer(0, 0));
        assert!(!tick_is_newer(0, 1));
        assert!(tick_is_newer(0, u32::MAX));
        assert!(tick_is_newer(5, u32::MAX - 3));
        assert!(!tick_is_newer(u32::MAX, 0));
    }

    #[test]
    fn test_event_channel_cursor_consumers() {
        let mut channel: EventChannel<u32> = EventChannel::new();
        channel.send(1);
        channel.send(2);
        let mut cursor_a = 0;
        let cursor_b = 0;
        assert_eq!(channel.events_since(cursor_a), &[1, 2]);
        cursor_a = channel.sequence();
        channel.send(3);
        assert_eq!(channel.events_since(cursor_a), &[3]);
        cursor_a = channel.sequence();
        assert_eq!(channel.events_since(cursor_b), &[1, 2, 3]);
        let cursor_b = channel.sequence();
        assert!(channel.events_since(cursor_a).is_empty());
        assert!(channel.events_since(cursor_b).is_empty());
    }

    #[test]
    fn test_event_channel_consume_is_exactly_once() {
        let mut channel: EventChannel<u32> = EventChannel::new();
        channel.send(1);
        channel.send(2);
        let mut cursor_a = 0;
        let mut cursor_b = 0;
        assert_eq!(channel.consume(&mut cursor_a), &[1, 2]);
        assert!(channel.consume(&mut cursor_a).is_empty());
        channel.update();
        channel.send(3);
        assert_eq!(
            channel.consume(&mut cursor_a),
            &[3],
            "the two-frame buffer must not re-deliver"
        );
        assert_eq!(channel.consume(&mut cursor_b), &[1, 2, 3]);
    }

    #[test]
    fn test_event_channel_two_frame_expiry() {
        let mut channel: EventChannel<u32> = EventChannel::new();
        channel.send(1);
        channel.update();
        assert_eq!(channel.len(), 1, "event survives its first frame boundary");
        channel.send(2);
        channel.update();
        assert_eq!(
            channel.events_since(0),
            &[2],
            "first event expired, second survives"
        );
        channel.update();
        assert!(channel.is_empty());
    }

    #[test]
    fn test_event_channel_read_frame_is_settled_prior_frame() {
        let mut channel: EventChannel<u32> = EventChannel::new();
        assert_eq!(channel.read_frame(), &[] as &[u32]);
        channel.send(1);
        channel.send(2);
        assert_eq!(
            channel.read_frame(),
            &[] as &[u32],
            "this frame's sends are not settled yet"
        );
        channel.update();
        assert_eq!(channel.read_frame(), &[1, 2]);
        channel.send(3);
        assert_eq!(
            channel.read_frame(),
            &[1, 2],
            "a later send does not change the settled frame"
        );
        assert_eq!(channel.read_frame(), &[1, 2], "a broadcast read repeats");
        channel.update();
        assert_eq!(
            channel.read_frame(),
            &[3],
            "frame one expired, frame two settled"
        );
        channel.update();
        assert_eq!(
            channel.read_frame(),
            &[] as &[u32],
            "an empty frame settles empty"
        );
    }

    #[test]
    fn test_event_channel_trim_and_lagging_cursor() {
        let mut channel: EventChannel<u32> = EventChannel::new();
        for value in 0..10 {
            channel.send(value);
        }
        channel.trim(4);
        assert_eq!(channel.len(), 6);
        assert_eq!(
            channel.events_since(0),
            &[4, 5, 6, 7, 8, 9],
            "a lagging cursor sees everything still buffered"
        );
        assert_eq!(channel.events_since(7), &[7, 8, 9]);
        assert_eq!(channel.sequence(), 10);
        channel.clear();
        assert!(channel.is_empty());
        assert_eq!(
            channel.sequence(),
            10,
            "clearing advances past events without reusing sequences"
        );
    }

    #[test]
    fn test_event_channel_capacity_backstop() {
        let mut channel: EventChannel<u32> = EventChannel::new();
        let total = EVENT_CHANNEL_CAPACITY as u32 + 10;
        for value in 0..total {
            channel.send(value);
        }
        assert_eq!(
            channel.len(),
            EVENT_CHANNEL_CAPACITY / 2 + 10,
            "crossing capacity must drop the oldest half exactly once"
        );
        assert_eq!(
            channel.sequence(),
            u64::from(total),
            "sequences must keep counting across the backstop"
        );
        assert_eq!(
            channel.peek(),
            Some(&(EVENT_CHANNEL_CAPACITY as u32 / 2)),
            "the survivor at the front is the first event after the dropped half"
        );
        assert_eq!(channel.events_since(u64::from(total) - 5).len(), 5);
        channel.update();
        assert_eq!(
            channel.len(),
            EVENT_CHANNEL_CAPACITY / 2 + 10,
            "a stale previous-update watermark behind the base must not over-trim"
        );
        channel.update();
        assert!(channel.is_empty());
        assert_eq!(channel.sequence(), u64::from(total));
    }

    #[test]
    fn test_stages_run_in_declaration_order() {
        let mut stages: Stages<Vec<&'static str>> = Stages::new();
        stages.add_stage("input").add_stage("simulation");
        stages.add_stage("render");
        stages
            .stage_mut("render")
            .push("draw", |log: &mut Vec<&'static str>| log.push("draw"));
        stages
            .stage_mut("input")
            .push("poll", |log: &mut Vec<&'static str>| log.push("poll"));
        stages
            .stage_mut("simulation")
            .push("physics", |log: &mut Vec<&'static str>| log.push("physics"))
            .push("ai", |log: &mut Vec<&'static str>| log.push("ai"));
        let mut log = Vec::new();
        stages.run(&mut log);
        assert_eq!(log, vec!["poll", "physics", "ai", "draw"]);
        log.clear();
        stages.run_stage("simulation", &mut log);
        assert_eq!(log, vec!["physics", "ai"]);
    }

    #[test]
    #[should_panic(expected = "already declared")]
    fn test_stages_reject_duplicate_names() {
        let mut stages: Stages<u32> = Stages::new();
        stages.add_stage("simulation").add_stage("simulation");
    }

    #[test]
    #[should_panic(expected = "declared stages are [\"input\"]")]
    fn test_stages_missing_stage_names_the_declared_set() {
        let mut stages: Stages<u32> = Stages::new();
        stages.add_stage("input");
        stages.stage_mut("simulation");
    }
}
