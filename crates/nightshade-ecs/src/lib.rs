pub use paste;

pub mod access;
pub use access::{Access, SystemAccess};

mod ecs;
pub use ecs::*;

pub mod system_param;

pub mod gate;

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

        impl $name {
            $vis fn builder() -> Self {
                Self {
                    $($field: ::core::default::Default::default()),+
                }
            }

            $(
                $vis fn $field(mut self, value: $ty) -> Self {
                    self.$field = value;
                    self
                }
            )+
        }

        impl $crate::SealedBundle for $name {}

        impl $crate::Bundle for $name {
            fn component_mask(world: &mut $crate::Storage) -> $crate::Mask {
                let mut mask = $crate::Mask::EMPTY;
                $(
                    let element_mask =
                        <$ty as $crate::Component>::component_mask(world);
                    assert!(
                        !mask.intersects(element_mask),
                        "bundle! structs must not repeat a component type"
                    );
                    mask |= element_mask;
                )+
                mask
            }

            fn lookup_mask(world: &$crate::Storage) -> $crate::Mask {
                let mut mask = $crate::Mask::EMPTY;
                $(mask |= <$ty as $crate::Component>::lookup_mask(world);)+
                mask
            }

            fn strict_mask(world: &$crate::Storage) -> Option<$crate::Mask> {
                let mut mask = $crate::Mask::EMPTY;
                $(mask |= <$ty as $crate::Component>::strict_mask(world)?;)+
                Some(mask)
            }

            fn write(self, world: &mut $crate::Storage, entity: $crate::Entity) {
                $(<$ty as $crate::Component>::write(self.$field, world, entity);)+
            }

        }

        impl $crate::CloneBundle for $name {
            fn stage(
                &self,
                entity: $crate::Entity,
                staged: &mut $crate::StagedComponents,
            ) {
                $(
                    <$ty as $crate::Component>::stage(&self.$field, entity, staged);
                )+
            }
        }
    };
}

#[cfg(not(target_family = "wasm"))]
pub use rayon;

mod archetype;
mod change;
mod entity;
mod events;
mod schedule;
mod tags;

pub(crate) use archetype::{
    ArchetypeEdges, ArchetypeRouting, PlannedTable, archetype_cached_tables,
    archetype_planned_tables, archetype_read_cached_tables, archetype_read_planned_tables,
    archetype_register_table,
};
pub use change::{STRUCTURAL_LOG_CAPACITY, StructuralChange, StructuralChangeKind, tick_is_newer};
pub use entity::{Entity, EntityAllocator, EntitySlot, IdCursor};
pub(crate) use entity::{EntityLocation, EntityLocations};
pub use events::{EVENT_CHANNEL_CAPACITY, EventChannel};
pub use schedule::{BatchLayout, Schedule, Stages, SystemRunner};
pub use tags::SparseTagSet;

#[cfg(test)]
mod tests;
