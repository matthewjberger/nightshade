use crate::commands::post_message;
use crate::data::ask::Ask;
use crate::data::rules::*;
use crate::data::theme;
use crate::resources::{BlackmarrowResources, Console, Screen};
use crate::tags::{Eyes, Player, Torch};
use nightshade::prelude::*;
use nightshade_flashlight::commands::{recharge, switch};
use nightshade_flashlight::{Battery, FlashlightSettings, Lit};
use nightshade_input::Actions;
use nightshade_physics::contract::components::{
    CharacterControllerComponent, PhysicsInterpolation,
};
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_senses::commands::{hide, make_a_noise};
use nightshade_senses::{Exposure, Noises, SensesSettings};
use nightshade_shell::{ShellState, shell_is_capturing_input};

pub fn update(
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    settings: Res<FlashlightSettings>,
    senses_settings: Res<SensesSettings>,
    shell: Res<ShellState<Console>>,
    mut resources: ResMut<BlackmarrowResources>,
    mut noises: ResMut<Noises>,
    mut bodies: Query<
        (
            &mut LocalTransform,
            &mut CharacterControllerComponent,
            &mut Exposure,
            Option<&mut PhysicsInterpolation>,
        ),
        WithTag<Player>,
    >,
    mut eyes: Query<(&GlobalTransform, &LocalTransform), WithTag<Eyes>>,
    mut torches: Query<(&mut Battery, &mut Lit), WithTag<Torch>>,
) {
    let resources = &mut *resources;
    if resources.screen != Screen::Playing {
        return;
    }
    let delta = time.delta_time;
    resources.run.clock += delta;
    resources.player.bite_cooldown = (resources.player.bite_cooldown - delta).max(0.0);

    let Some(body) = resources.player.body else {
        return;
    };

    let console_open = shell_is_capturing_input(&shell);
    let noclip = resources.player.noclip;
    let flight = if noclip && !console_open {
        flight_step(
            &actions,
            resources.player.forward,
            delta,
            shell.context.fly_speed,
        )
    } else {
        Vec3::zeros()
    };

    let mut stance = None;
    bodies.for_entity(body, |(transform, controller, exposure, interpolation)| {
        controller.max_speed = WALK_SPEED;
        controller.disable_gravity = noclip;
        controller.engine_input_enabled = !noclip && !console_open;
        if noclip {
            transform.translation += flight;
            if let Some(interpolation) = interpolation {
                interpolation.previous_translation = transform.translation;
                interpolation.current_translation = transform.translation;
            }
        }
        stance = Some((
            transform.translation,
            controller.is_crouching,
            controller.is_sprinting,
            controller.velocity.xz().magnitude(),
        ));
        hide(
            exposure,
            &senses_settings,
            controller.is_crouching,
            controller.is_sprinting,
        );
    });
    let Some((position, crouching, sprinting, speed)) = stance else {
        return;
    };
    resources.player.crouching = crouching;
    resources.player.sprinting = sprinting;
    resources.player.moving = speed > 0.35;
    resources.player.at = position;

    let feet = position - Vec3::new(0.0, 1.0, 0.0);
    if let Some(eye_entity) = resources.player.eyes {
        eyes.for_entity(eye_entity, |(global, local)| {
            resources.player.eye = global.translation();
            resources.player.forward = nalgebra_glm::normalize(&global.forward_vector());
            resources.player.head = local.rotation;
        });
    }

    step_noise(resources, &mut noises, feet, delta, sprinting, crouching);

    if actions.just_pressed(Ask::Torch) && !console_open {
        let mut flat = false;
        let mut used_cell = false;
        torches.for_each_mut(|_entity, (battery, lit)| {
            if battery.0 <= 0.0 {
                if resources.player.spare_cells > 0 {
                    resources.player.spare_cells -= 1;
                    recharge(battery, &settings, CELL_REFILL);
                    used_cell = true;
                } else {
                    flat = true;
                }
                return;
            }
            switch(lit, battery);
        });
        if used_cell {
            post_message(
                &mut resources.run,
                "You put a fresh cell in. The beam comes back.",
            );
        } else if flat {
            post_message(
                &mut resources.run,
                "The cell is dead and you have no other.",
            );
        }
    }

    if (feet - theme::EXIT_AT).magnitude() < EXIT_RADIUS {
        resources.pending_screen = Some(Screen::Escaped);
    }
    if resources.player.health <= 0.0 {
        resources.pending_screen = Some(Screen::Dead);
    }
}

fn flight_step(actions: &Actions<Ask>, forward: Vec3, delta: f32, speed: f32) -> Vec3 {
    let right = nalgebra_glm::normalize(&nalgebra_glm::cross(&forward, &Vec3::y()));
    let asked = actions.axis2(Ask::Fly);
    let mut direction = forward * asked.y + right * asked.x;
    if actions.pressed(Ask::Rise) {
        direction += Vec3::y();
    }
    if actions.pressed(Ask::Sink) {
        direction -= Vec3::y();
    }
    if direction.magnitude() < 0.001 {
        return Vec3::zeros();
    }
    let haste = if actions.pressed(Ask::Haste) {
        2.0
    } else {
        1.0
    };
    nalgebra_glm::normalize(&direction) * speed * haste * delta
}

fn step_noise(
    resources: &mut BlackmarrowResources,
    noises: &mut Noises,
    feet: Vec3,
    delta: f32,
    sprinting: bool,
    crouching: bool,
) {
    if !resources.player.moving {
        resources.player.footfall = 0.0;
        return;
    }
    resources.player.footfall += delta;
    let interval = if sprinting {
        FOOTFALL_INTERVAL * 0.62
    } else if crouching {
        FOOTFALL_INTERVAL * 1.5
    } else {
        FOOTFALL_INTERVAL
    };
    if resources.player.footfall < interval {
        return;
    }
    resources.player.footfall = 0.0;
    let loudness = if crouching {
        NOISE_CROUCH
    } else if sprinting {
        NOISE_SPRINT
    } else {
        NOISE_FOOTFALL
    };
    make_a_noise(noises, feet, loudness);
}
