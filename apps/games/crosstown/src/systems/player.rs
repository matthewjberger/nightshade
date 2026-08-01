use crate::commands::pickups::throw_weapon;
use crate::commands::punch;
use crate::components::{
    Ammo, Combo, Dash, Facing, Fling, Holding, Leap, Lift, Pose, Rise, Step, Stride, Strike, Stun,
    Tap, Toward, Wish,
};
use crate::contract::{CrosstownCommand, CrosstownCommands};
use crate::data;
use crate::data::Ask;
use crate::queries::{hero_limit, run_pose, tapped, walk, walk_pose};
use crate::resources::{Ring, Sheets};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;

#[derive(Clone, Copy)]
struct Order {
    time: f32,
    pose: u32,
    step: u32,
    combo: f32,
    lunge: f32,
    spend: bool,
}

pub fn ask(
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    mut queue: ResMut<CrosstownCommands>,
    heroes: Intents,
) {
    if actions.just_pressed(Ask::Punch) {
        punch(&mut queue);
    }
    let asked = std::mem::take(&mut queue.pending).contains(&CrosstownCommand::Punch);
    let leapt = actions.just_pressed(Ask::Jump);
    let tap = tapped(&actions);
    let delta_time = time.delta_time;
    heroes.for_each(
        |_entity, (wish, leap, window, toward, dash, stun, strike)| {
            wish.0 = (wish.0 - delta_time).max(0.0);
            leap.0 = (leap.0 - delta_time).max(0.0);
            window.0 = (window.0 - delta_time).max(0.0);
            if asked {
                wish.0 = data::BUFFER_TIME;
            }
            if leapt {
                leap.0 = data::BUFFER_TIME;
            }
            if tap == 0.0 {
                return;
            }
            if window.0 > 0.0 && toward.0 == tap && stun.0 <= 0.0 && strike.0 <= 0.0 {
                dash.0 = 1.0;
                window.0 = 0.0;
                return;
            }
            window.0 = data::TAP_WINDOW;
            toward.0 = tap;
        },
    );
}

pub fn fight(time: Res<Time>, ring: Res<Ring>, mut heroes: Fists) {
    let delta_time = if ring.hit_stop > 0.0 {
        0.0
    } else {
        time.delta_time
    };
    let mut order = None;
    heroes.state.for_each_mut(
        |_entity, (strike, stun, holding, lift, dash, combo, step, facing)| {
            if stun.0 > 0.0 || strike.0 > data::MOVE_CANCEL {
                return;
            }
            let facing = facing.0;
            order = Some(if holding.0 != 0 {
                Order {
                    time: data::SWING_TIME,
                    pose: data::POSE_SWING,
                    step: step.0,
                    combo: combo.0,
                    lunge: facing * data::SWING_LUNGE,
                    spend: true,
                }
            } else if lift.0 > 0.0 {
                Order {
                    time: data::BLOW_TIME * 1.4,
                    pose: data::POSE_AIR_KICK,
                    step: 3,
                    combo: combo.0,
                    lunge: facing * data::AIR_LUNGE,
                    spend: false,
                }
            } else if dash.0 > 0.0 {
                Order {
                    time: data::DASH_BLOW_TIME,
                    pose: data::POSE_SLIDE,
                    step: 4,
                    combo: 0.0,
                    lunge: facing * data::DASH_LUNGE,
                    spend: false,
                }
            } else {
                let next = if combo.0 > 0.0 { (step.0 + 1) % 3 } else { 0 };
                Order {
                    time: data::BLOW_TIME,
                    pose: data::HERO_BLOWS[next as usize],
                    step: next,
                    combo: data::COMBO_TIME + data::BLOW_TIME,
                    lunge: facing * data::HERO_LUNGE[next as usize],
                    spend: false,
                }
            });
        },
    );
    heroes.act.for_each_mut(
        |_entity, (strike, step, combo, pose, holding, ammo, wish, fling)| {
            combo.0 = (combo.0 - delta_time).max(0.0);
            if wish.0 <= 0.0 {
                return;
            }
            let Some(order) = order else {
                return;
            };
            wish.0 = 0.0;
            strike.0 = order.time;
            step.0 = order.step;
            combo.0 = order.combo;
            pose.0 = order.pose;
            fling.0 = order.lunge;
            if !order.spend {
                return;
            }
            ammo.0 = ammo.0.saturating_sub(1);
            if ammo.0 == 0 {
                holding.0 = 0;
            }
        },
    );
}

pub fn hop(
    actions: Res<Actions<Ask>>,
    ring: Res<Ring>,
    sheets: Res<Sheets>,
    mut commands: Commands,
    heroes: Hops,
) {
    let slinging = actions.just_pressed(Ask::Throw);
    let feet = ring.feet;
    let stage = ring.piece;
    let mut thrown = None;
    heroes.for_each(
        |_entity, (lift, rise, pose, holding, leap, strike, stun, facing)| {
            let busy = stun.0 > 0.0;
            if !busy && slinging && holding.0 != 0 {
                thrown = Some((holding.0 - 1, facing.0));
                holding.0 = 0;
                pose.0 = data::POSE_GRAB;
                return;
            }
            if !busy && leap.0 > 0.0 && lift.0 <= 0.0 {
                leap.0 = 0.0;
                rise.0 = data::JUMP_SPEED;
                pose.0 = data::POSE_JUMP;
                return;
            }
            if strike.0 <= 0.0 && lift.0 > 0.0 {
                pose.0 = if rise.0 > 0.0 {
                    data::POSE_JUMP
                } else {
                    data::POSE_FALL
                };
            }
        },
    );
    if let Some((kind, facing)) = thrown {
        throw_weapon(&mut commands, &sheets, kind, feet, facing, stage);
    }
}

pub fn stroll(
    time: Res<Time>,
    actions: Res<Actions<Ask>>,
    mut ring: ResMut<Ring>,
    mut heroes: Walkers,
) {
    let direction = walk(&actions);
    let delta_time = if ring.hit_stop > 0.0 {
        0.0
    } else {
        time.delta_time
    };
    let (low, high) = hero_limit(
        ring.camera_x,
        ring.cleared,
        ring.gate_x,
        ring.end_x,
        ring.last_wave,
    );
    let mut feet = ring.feet;
    heroes.feet.for_each_mut(
        |_entity, (facing, stride, transform, dash, strike, stun, lift, toward)| {
            let held = strike.0 > data::MOVE_CANCEL || stun.0 > 0.0;
            if held || direction.x * toward.0 <= 0.0 {
                dash.0 = 0.0;
            }
            let running = dash.0 > 0.0;
            let moving = direction.magnitude() > 0.0 && !held;
            if !moving {
                if !held && lift.0 <= 0.0 {
                    stride.0 = 0.0;
                }
            } else {
                let control = if lift.0 > 0.0 { data::AIR_CONTROL } else { 1.0 };
                let (speed, depth, rate) = if running {
                    (data::HERO_RUN, data::HERO_RUN_DEPTH, data::RUN_STRIDE_RATE)
                } else {
                    (data::HERO_SPEED, data::HERO_DEPTH_SPEED, data::STRIDE_RATE)
                };
                facing.0 = if running {
                    toward.0
                } else if direction.x != 0.0 {
                    direction.x.signum()
                } else {
                    facing.0
                };
                transform.translation.x += direction.x * speed * delta_time * control;
                transform.translation.z += direction.y * depth * delta_time * control;
                stride.0 += delta_time * rate;
            }
            transform.translation.x = transform.translation.x.clamp(low, high);
            transform.translation.z = transform
                .translation
                .z
                .clamp(-data::LANE_HALF, data::LANE_HALF);
            feet = transform.translation;
        },
    );
    heroes.look.for_each_mut(
        |_entity, (pose, stride, dash, strike, stun, lift, holding)| {
            if strike.0 > 0.0 || stun.0 > 0.0 || lift.0 > 0.0 {
                return;
            }
            pose.0 = if stride.0 <= 0.0 {
                if holding.0 == 0 {
                    data::POSE_IDLE
                } else {
                    data::POSE_HOLD
                }
            } else if holding.0 != 0 {
                data::POSE_HOLD
            } else if dash.0 > 0.0 {
                run_pose(stride.0)
            } else {
                walk_pose(stride.0)
            };
        },
    );
    ring.feet = feet;
}

pub type Intents<'world, 'item> = Query<
    'world,
    (
        &'item mut Wish,
        &'item mut Leap,
        &'item mut Tap,
        &'item mut Toward,
        &'item mut Dash,
        &'item Stun,
        &'item Strike,
    ),
    With<Combo>,
>;

pub type Hops<'world, 'item> = Query<
    'world,
    (
        &'item mut Lift,
        &'item mut Rise,
        &'item mut Pose,
        &'item mut Holding,
        &'item mut Leap,
        &'item Strike,
        &'item Stun,
        &'item Facing,
    ),
>;

pub type HeroState<'world, 'item> = Query<
    'world,
    (
        &'item Strike,
        &'item Stun,
        &'item Holding,
        &'item Lift,
        &'item Dash,
        &'item Combo,
        &'item Step,
        &'item Facing,
    ),
>;

pub type HeroAct<'world, 'item> = Query<
    'world,
    (
        &'item mut Strike,
        &'item mut Step,
        &'item mut Combo,
        &'item mut Pose,
        &'item mut Holding,
        &'item mut Ammo,
        &'item mut Wish,
        &'item mut Fling,
    ),
>;

nightshade::ecs::query_bundle! {
    pub struct Fists<'world, 'item> {
        pub state: HeroState<'world, 'item>,
        pub act: HeroAct<'world, 'item>,
    }
}

pub type HeroFeet<'world, 'item> = Query<
    'world,
    (
        &'item mut Facing,
        &'item mut Stride,
        &'item mut LocalTransform,
        &'item mut Dash,
        &'item Strike,
        &'item Stun,
        &'item Lift,
        &'item Toward,
    ),
>;

pub type HeroLook<'world, 'item> = Query<
    'world,
    (
        &'item mut Pose,
        &'item Stride,
        &'item Dash,
        &'item Strike,
        &'item Stun,
        &'item Lift,
        &'item Holding,
    ),
    With<Combo>,
>;

nightshade::ecs::query_bundle! {
    pub struct Walkers<'world, 'item> {
        pub feet: HeroFeet<'world, 'item>,
        pub look: HeroLook<'world, 'item>,
    }
}
