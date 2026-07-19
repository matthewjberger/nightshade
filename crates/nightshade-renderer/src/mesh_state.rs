//! Per-frame mesh dirty-tracking state shared between the caller and the
//! mesh passes: dirty sets, add/remove queues, and rebuild flags.

use std::collections::{HashMap, HashSet};

use nightshade_ecs::Entity;

/// Accumulates the frame's mesh dirty state and hands it to the passes.
#[derive(Debug, Default)]
pub struct MeshRenderState {
    /// The dirty state accumulated for the current frame.
    pub inner: MeshRenderStateInner,
}

/// The dirty sets, queues, and flags describing what changed this frame.
#[derive(Debug, Default)]
pub struct MeshRenderStateInner {
    /// Entities whose world transform changed this frame.
    pub transform_dirty: HashSet<Entity>,
    /// Entities whose material assignment changed this frame.
    pub material_dirty: HashSet<Entity>,
    /// Instanced entities whose local instance matrices changed this frame.
    pub instanced_local_matrices_dirty: HashSet<Entity>,
    /// Per-entity custom data overrides, RGBA with alpha driving fade.
    pub custom_data_overrides: HashMap<Entity, [f32; 4]>,
    /// Entities whose custom data changed this frame.
    pub custom_data_dirty: HashSet<Entity>,
    /// Entities whose non-structural per-object state (visibility, morph
    /// weights) changed this frame. The pass refreshes exactly these slots
    /// instead of scanning every registered object.
    pub object_state_dirty: HashSet<Entity>,
    /// The instanced-transform compute pass must dispatch this frame.
    pub instanced_compute_dispatch_needed: bool,
    /// The set of instanced meshes changed this frame.
    pub instanced_meshes_changed: bool,
    /// Entities registered with the passes this frame, in order.
    pub entities_added: Vec<Entity>,
    /// Entities removed from the passes this frame, in order.
    pub entities_removed: Vec<Entity>,
    /// Draw batches must be rebuilt this frame.
    pub batches_invalidated: bool,
    /// The passes must rebuild every registered object this frame.
    pub full_rebuild_needed: bool,
    /// This state has been through at least one frame.
    pub frame_initialized: bool,
}

impl MeshRenderState {
    /// Creates a state whose first frame forces a full rebuild.
    pub fn new() -> Self {
        Self {
            inner: MeshRenderStateInner {
                transform_dirty: HashSet::new(),
                material_dirty: HashSet::new(),
                instanced_local_matrices_dirty: HashSet::new(),
                custom_data_overrides: HashMap::new(),
                custom_data_dirty: HashSet::new(),
                object_state_dirty: HashSet::new(),
                instanced_compute_dispatch_needed: false,
                instanced_meshes_changed: false,
                entities_added: Vec::new(),
                entities_removed: Vec::new(),
                batches_invalidated: false,
                full_rebuild_needed: true,
                frame_initialized: false,
            },
        }
    }

    /// Marks `entity`'s transform changed this frame.
    pub fn mark_transform_dirty(&mut self, entity: Entity) {
        self.inner.transform_dirty.insert(entity);
    }

    /// Marks `entity`'s per-object state (visibility, morph weights) changed.
    pub fn mark_object_state_dirty(&mut self, entity: Entity) {
        self.inner.object_state_dirty.insert(entity);
    }

    /// Marks `entity`'s instanced local matrices changed this frame.
    pub fn mark_instanced_local_matrices_dirty(&mut self, entity: Entity) {
        self.inner.instanced_local_matrices_dirty.insert(entity);
    }

    /// Requests the instanced-transform compute pass dispatch this frame.
    pub fn mark_instanced_compute_dispatch_needed(&mut self) {
        self.inner.instanced_compute_dispatch_needed = true;
    }

    /// Marks the set of instanced meshes changed this frame.
    pub fn mark_instanced_meshes_changed(&mut self) {
        self.inner.instanced_meshes_changed = true;
    }

    /// Sets `entity`'s custom data, invalidating batches when the alpha
    /// crosses the opaque threshold.
    pub fn set_entity_custom_data(&mut self, entity: Entity, data: [f32; 4]) {
        let old_alpha = self
            .inner
            .custom_data_overrides
            .get(&entity)
            .map_or(1.0, |d| d[3]);
        if (old_alpha < 1.0) != (data[3] < 1.0) {
            self.inner.batches_invalidated = true;
        }
        self.inner.custom_data_overrides.insert(entity, data);
        self.inner.custom_data_dirty.insert(entity);
    }

    /// Resets `entity`'s custom data to fully opaque and invalidates batches.
    pub fn mark_entity_fade_complete(&mut self, entity: Entity) {
        self.inner
            .custom_data_overrides
            .insert(entity, [1.0, 1.0, 1.0, 1.0]);
        self.inner.custom_data_dirty.insert(entity);
        self.inner.batches_invalidated = true;
    }

    /// Requests a full rebuild of every registered object next frame.
    pub fn request_full_rebuild(&mut self) {
        self.inner.full_rebuild_needed = true;
    }

    /// Marks `entity`'s material changed and invalidates batches.
    pub fn mark_material_dirty(&mut self, entity: Entity) {
        self.inner.material_dirty.insert(entity);
        self.inner.batches_invalidated = true;
    }

    /// Queues `entity` for registration and invalidates batches.
    pub fn mark_entity_added(&mut self, entity: Entity) {
        self.inner.entities_added.push(entity);
        self.inner.batches_invalidated = true;
    }

    /// Queues `entity` for removal, invalidates batches, and drops its
    /// pending dirty marks.
    pub fn mark_entity_removed(&mut self, entity: Entity) {
        self.inner.entities_removed.push(entity);
        self.inner.batches_invalidated = true;
        self.inner.transform_dirty.remove(&entity);
        self.inner.material_dirty.remove(&entity);
    }

    /// Returns true when any dirty set, queue, or rebuild flag is active.
    pub fn has_any_changes(&self) -> bool {
        !self.inner.transform_dirty.is_empty()
            || !self.inner.material_dirty.is_empty()
            || !self.inner.entities_added.is_empty()
            || !self.inner.entities_removed.is_empty()
            || self.inner.batches_invalidated
            || self.inner.full_rebuild_needed
    }

    /// Returns entities whose transforms, materials, or per-object state
    /// (visibility, morph weights) changed this frame. Used by the
    /// per-camera dirty-tracking path to decide which viewports need to
    /// re-render based on frustum overlap with the changed entities'
    /// bounding spheres.
    pub fn dirty_entities_for_culling(&self) -> impl Iterator<Item = Entity> + '_ {
        self.inner
            .transform_dirty
            .iter()
            .chain(self.inner.material_dirty.iter())
            .chain(self.inner.entities_added.iter())
            .chain(self.inner.object_state_dirty.iter())
            .copied()
    }

    /// Returns true when the renderer cannot localize dirty work to a
    /// single viewport's frustum (e.g. a full rebuild, batches were
    /// invalidated, or entities were removed). Cached viewports must
    /// re-render when this is true.
    pub fn requires_full_invalidation(&self) -> bool {
        self.inner.full_rebuild_needed
            || self.inner.batches_invalidated
            || !self.inner.entities_removed.is_empty()
    }

    /// Returns whether this state has been through at least one frame.
    pub fn frame_initialized(&self) -> bool {
        self.inner.frame_initialized
    }

    /// Returns whether a full rebuild is pending.
    pub fn full_rebuild_needed(&self) -> bool {
        self.inner.full_rebuild_needed
    }

    /// Returns whether draw batches are invalidated this frame.
    pub fn batches_invalidated(&self) -> bool {
        self.inner.batches_invalidated
    }

    /// Moves the accumulated frame state out, leaving a cleared state marked
    /// as initialized.
    pub fn take_frame_state(&mut self) -> MeshRenderStateInner {
        std::mem::replace(
            &mut self.inner,
            MeshRenderStateInner {
                frame_initialized: true,
                ..Default::default()
            },
        )
    }

    /// Merges a taken-but-never-consumed frame state back in, so dirty
    /// marks captured for a frame that skipped rendering survive into the
    /// next frame instead of being dropped.
    pub fn restore_unconsumed(&mut self, unconsumed: MeshRenderStateInner) {
        let newer = std::mem::replace(&mut self.inner, unconsumed);
        self.inner.merge_newer(newer);
    }
}

impl MeshRenderStateInner {
    /// Folds a newer frame's dirty state into this one: sets union, flags
    /// combine, and add/remove orders keep this state's entries first so
    /// chronology is preserved.
    pub fn merge_newer(&mut self, newer: MeshRenderStateInner) {
        self.transform_dirty.extend(newer.transform_dirty);
        self.material_dirty.extend(newer.material_dirty);
        self.instanced_local_matrices_dirty
            .extend(newer.instanced_local_matrices_dirty);
        self.custom_data_overrides
            .extend(newer.custom_data_overrides);
        self.custom_data_dirty.extend(newer.custom_data_dirty);
        self.object_state_dirty.extend(newer.object_state_dirty);
        self.instanced_compute_dispatch_needed |= newer.instanced_compute_dispatch_needed;
        self.instanced_meshes_changed |= newer.instanced_meshes_changed;
        self.entities_added.extend(newer.entities_added);
        self.entities_removed.extend(newer.entities_removed);
        self.batches_invalidated |= newer.batches_invalidated;
        self.full_rebuild_needed |= newer.full_rebuild_needed;
        self.frame_initialized |= newer.frame_initialized;
    }
}
