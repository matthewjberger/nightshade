use crate::contract::render_data::{SdfEditInstance, SdfFrame};
use crate::contract::{
    SdfBlend, SdfCollides, SdfMetal, SdfOperation, SdfOrder, SdfRoughness, SdfShape, SdfTint,
};
use nightshade::prelude::*;
use nightshade_scene::contract::GlobalTransform;
use nightshade_scene::contract::primitives::Visibility;
use std::collections::HashMap;

pub type SdfEdits<'world, 'item> = Query<
    'world,
    (
        &'item SdfShape,
        &'item GlobalTransform,
        &'item SdfOperation,
        &'item SdfBlend,
        &'item SdfTint,
        &'item SdfOrder,
        Option<&'item Visibility>,
        Option<&'item SdfCollides>,
    ),
>;

pub type SdfSurfaces<'world, 'item> = Query<
    'world,
    (
        &'item SdfShape,
        Option<&'item SdfRoughness>,
        Option<&'item SdfMetal>,
    ),
>;

pub fn gather_edits_system(mut frame: ResMut<SdfFrame>, edits: SdfEdits, surfaces: SdfSurfaces) {
    frame.edits.clear();
    let mut finish: HashMap<Entity, (f32, f32)> = HashMap::new();
    surfaces.for_each(|entity, (_, roughness, metal)| {
        finish.insert(
            entity,
            (
                roughness.copied().unwrap_or_default().0,
                metal.copied().unwrap_or_default().0,
            ),
        );
    });
    let mut ordered: Vec<(u32, SdfEditInstance)> = Vec::new();
    edits.for_each(
        |entity, (shape, global_transform, operation, blend, tint, order, visibility, collides)| {
            let visible = visibility.is_none_or(|visibility| visibility.visible);
            let (roughness, metal) = finish
                .get(&entity)
                .copied()
                .unwrap_or((SdfRoughness::default().0, 0.0));
            ordered.push((
                order.0,
                SdfEditInstance {
                    entity,
                    shape: *shape,
                    operation: *operation,
                    blend: blend.0,
                    tint: tint.0,
                    roughness,
                    metal,
                    transform: global_transform.0,
                    visible,
                    collides: collides.is_none_or(|collides| collides.0),
                },
            ));
        },
    );
    ordered.sort_by_key(|(order, _)| *order);
    frame
        .edits
        .extend(ordered.into_iter().map(|(_, instance)| instance));
}
