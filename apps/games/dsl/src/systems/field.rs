use crate::resources::{Machine, Script, Store};
use nightshade::prelude::*;
use nightshade_sdf::contract::{SdfBlend, SdfOperation, SdfOrder, SdfShape, SdfTint};

pub fn shape(name: &str, size: Vec3) -> SdfShape {
    match name {
        "cube" => SdfShape::Cube {
            half_extents: size * 0.5,
            rounding: 0.0,
        },
        "capsule" => SdfShape::Capsule {
            half_height: size.y * 0.5,
            radius: size.x * 0.5,
        },
        "torus" => SdfShape::Torus {
            major_radius: size.x * 0.5,
            minor_radius: size.y * 0.25,
        },
        "cylinder" => SdfShape::Cylinder {
            half_height: size.y * 0.5,
            radius: size.x * 0.5,
        },
        _ => SdfShape::Sphere {
            radius: size.x * 0.5,
        },
    }
}

pub fn carve(
    script: Res<Script>,
    machine: Res<Machine>,
    mut store: ResMut<Store>,
    mut commands: Commands,
) {
    let known = &script.plan.known;
    let (Some(field), Some(place)) = (known.field, known.transform) else {
        return;
    };
    if store.carved == store.shape {
        return;
    }
    store.carved = store.shape;
    let width = store.rows.len() as u32;
    for row in 0..width {
        if !store.living(row) || !store.holds(field, row) {
            continue;
        }
        let Some(entity) = store.entity(row) else {
            continue;
        };
        let name = crate::queries::show(&store, &machine, store.read(field, row));
        let size = match known.scale.filter(|key| store.holds(*key, row)) {
            Some(key) => {
                let held = store.read(key, row);
                Vec3::new(held.axis(0), held.axis(1), held.axis(2))
            }
            None => Vec3::new(1.0, 1.0, 1.0),
        };
        let tone = match known.color.filter(|key| store.holds(*key, row)) {
            Some(key) => {
                let held = store.read(key, row);
                Vec3::new(held.axis(0), held.axis(1), held.axis(2))
            }
            None => Vec3::new(0.72, 0.7, 0.66),
        };
        let cut = known
            .carve
            .filter(|key| store.holds(*key, row))
            .map(|key| store.read(key, row).value() > 0.5)
            .unwrap_or(false);
        let blend = known
            .blend
            .filter(|key| store.holds(*key, row))
            .map(|key| store.read(key, row).value())
            .unwrap_or(0.0);
        commands.set(entity, shape(&name, size));
        commands.set(
            entity,
            match cut {
                true => SdfOperation::Subtract,
                false => SdfOperation::Union,
            },
        );
        commands.set(entity, SdfBlend(blend));
        commands.set(entity, SdfTint(tone));
        commands.set(entity, SdfOrder(row));
        let _ = place;
    }
}
