use crate::data;
use crate::data::arsenal::gun;
use crate::resources::BoomstickResources;
use crate::tags::Viewmodel;
use nightshade::prelude::*;
use nightshade_scene::prelude::*;

pub fn carry(mut resources: ResMut<BoomstickResources>, mut commands: Commands) {
    let wanted = resources.player.gun;
    let Some(eyes) = resources.player.eyes else {
        return;
    };
    if resources.player.viewmodel.is_some() && resources.shown.gun == wanted {
        return;
    }
    if let Some(old) = resources.player.viewmodel.take() {
        commands.despawn_recursive(old);
    }
    let model = crate::commands::build_parts(
        &mut commands,
        &format!("boomstick_gun_{wanted}"),
        gun(wanted).parts,
        data::VIEWMODEL_AT,
        data::VIEWMODEL_SCALE,
    );
    commands.set(model, nightshade::ecs::ChildOf(eyes));
    commands.add_tag_type::<Viewmodel>(model);
    resources.player.viewmodel = Some(model);
    resources.shown.gun = wanted;
}

pub fn hold(
    resources: Res<BoomstickResources>,
    models: Query<(&mut LocalTransform,), WithTag<Viewmodel>>,
) {
    let player = &resources.player;
    let sway = player.sway * data::VIEWMODEL_SWAY;
    let punch = player.punch;
    models.for_each(|_entity, (transform,)| {
        transform.translation =
            data::VIEWMODEL_AT + Vec3::new(-sway.x, -sway.y - punch * 0.3, punch * 0.55);
        transform.rotation = nalgebra_glm::quat_angle_axis(punch * 1.3, &Vec3::x());
    });
}
