use crate::data::cell::Tag;
use crate::resources::{Machine, Script, Store};
use nightshade::prelude::*;
use nightshade_graphics::prelude::*;
use nightshade_render::contract::Light;
use nightshade_scene::prelude::LocalTransform;

pub fn settle(
    script: Res<Script>,
    machine: Res<Machine>,
    mut store: ResMut<Store>,
    mut draw: ResMut<DebugDraw>,
    mut settings: ResMut<RenderSettings>,
    mut transforms: Query<(&mut LocalTransform,)>,
    mut lights: Query<(&mut Light,)>,
) {
    if store.fresh {
        store.fresh = false;
        settings.atmosphere = Atmosphere::None;
        settings.show_sky = false;
        settings.clear_color = crate::data::CLEAR_COLOR;
        settings.ibl_blend_factor = 0.0;
        draw.show_grid = false;
        if let Some(sun) = store.sun {
            let turn = nalgebra_glm::quat_angle_axis(crate::data::SUN_PITCH, &Vec3::x());
            transforms.for_entity(sun, |(transform,)| {
                transform.rotation = turn;
            });
            lights.for_entity(sun, |(light,)| {
                let tone = crate::data::SUN_TONE;
                light.color = Vec3::new(tone[0], tone[1], tone[2]);
                light.intensity = crate::data::SUN_STRENGTH;
                light.cast_shadows = false;
            });
        }
    }
    let known = &script.plan.known;
    if let Some(slot) = known.sky_show {
        let value = store.field(slot);
        if value.tag != Tag::Nothing {
            let show = value.truthy();
            settings.show_sky = show;
            settings.atmosphere = if show {
                Atmosphere::Sky
            } else {
                Atmosphere::None
            };
            settings.ibl_blend_factor = if show { 1.0 } else { 0.0 };
        }
    }
    if let Some(slot) = known.sky_look {
        let value = store.field(slot);
        if value.tag == Tag::Text {
            let name = crate::queries::show(&store, &machine, value);
            let wanted = match name.as_str() {
                "none" => Some(Atmosphere::None),
                "sky" => Some(Atmosphere::Sky),
                "clouds" => Some(Atmosphere::CloudySky),
                "space" => Some(Atmosphere::Space),
                "nebula" => Some(Atmosphere::Nebula),
                "sunset" => Some(Atmosphere::Sunset),
                "daynight" => Some(Atmosphere::DayNight),
                _ => None,
            };
            if let Some(wanted) = wanted {
                settings.atmosphere = wanted;
                let lit = wanted != Atmosphere::None;
                settings.show_sky = lit;
                settings.ibl_blend_factor = if lit { 1.0 } else { 0.0 };
            }
        }
    }
    if let Some(slot) = known.sky_grid {
        let value = store.field(slot);
        if value.tag != Tag::Nothing {
            draw.show_grid = value.truthy();
        }
    }
    if let Some(slot) = known.sky_ground {
        let value = store.field(slot);
        if value.tag == Tag::Vector {
            settings.clear_color = [value.axis(0), value.axis(1), value.axis(2), 1.0];
        }
    }

    let Some(sun) = store.sun else {
        return;
    };
    if let Some(slot) = known.sun_angle {
        let value = store.field(slot);
        if value.tag == Tag::Vector {
            let turn = nalgebra_glm::quat_angle_axis(value.axis(1), &Vec3::y())
                * nalgebra_glm::quat_angle_axis(value.axis(0), &Vec3::x());
            transforms.for_entity(sun, |(transform,)| {
                transform.rotation = turn;
            });
        }
    }
    let tone = known.sun_color.map(|slot| store.field(slot));
    let strength = known.sun_strength.map(|slot| store.field(slot));
    let shadows = known.sun_shadows.map(|slot| store.field(slot));
    lights.for_entity(sun, |(light,)| {
        if let Some(value) = tone
            && value.tag == Tag::Vector
        {
            light.color = Vec3::new(value.axis(0), value.axis(1), value.axis(2));
        }
        if let Some(value) = strength
            && value.tag == Tag::Number
        {
            light.intensity = value.value();
        }
        if let Some(value) = shadows
            && value.tag != Tag::Nothing
        {
            light.cast_shadows = value.truthy();
        }
    });
}
