use crate::contract::Dimension;
use crate::data;
use crate::data::Ask;
use crate::queries;
use crate::resources::{Arguments, Drive, FourfoldResources, Screen};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_state::prelude::{NextState, State};

pub fn arm(arguments: Res<Arguments>, mut resources: ResMut<FourfoldResources>) {
    resources.autoplay.on = arguments.autoplay;
}

pub fn toggle(actions: Res<Actions<Ask>>, mut resources: ResMut<FourfoldResources>) {
    if !actions.just_pressed(Ask::Autoplay) {
        return;
    }
    resources.autoplay.on = !resources.autoplay.on;
    resources.autoplay.step = 0;
    resources.autoplay.held = 0.0;
    resources.autoplay.jumps = 0;
    resources.autoplay.moves = 0;
}

pub fn open_the_door(
    resources: Res<FourfoldResources>,
    screen: Res<State<Screen>>,
    mut next: ResMut<NextState<Screen>>,
) {
    if !resources.autoplay.on || screen.current != Screen::Title {
        return;
    }
    next.pending = Some(Screen::Playing);
}

pub fn play(
    time: Res<Time>,
    dimension: Res<State<Dimension>>,
    mut resources: ResMut<FourfoldResources>,
) {
    if !resources.autoplay.on {
        return;
    }
    let delta_time = time.delta_time.min(0.05);
    resources.autoplay.held += delta_time;
    let here = dimension.current;
    let mut drive = Drive::default();

    match resources.autoplay.step {
        0 => {
            drive = travel(
                &mut resources,
                here,
                Dimension::Glass,
                data::AUTO_ARCH,
                delta_time,
            )
        }
        1 => {
            let arrived = resources.runner.position.x >= data::AUTO_GLASS_TOP;
            if here != Dimension::Glass || arrived {
                advance(&mut resources);
            } else {
                drive.wish = Vec2::new(1.0, 0.0);
                drive.jump = wants_jump(&mut resources);
            }
        }
        2 => {
            if resources.autoplay.held >= data::AUTO_LOOK_SECONDS {
                advance(&mut resources);
            }
        }
        3 => {
            if here != Dimension::Glass {
                advance(&mut resources);
            } else {
                drive.wish = Vec2::new(-1.0, 0.0);
            }
        }
        4 => {
            drive = travel(
                &mut resources,
                here,
                Dimension::Loom,
                data::AUTO_LOOM,
                delta_time,
            )
        }
        5 => {
            if here != Dimension::Loom {
                advance(&mut resources);
            } else {
                drive.wish = next_move(&mut resources);
            }
        }
        6 => {
            drive = travel(
                &mut resources,
                here,
                Dimension::Cairn,
                data::AUTO_CAIRN,
                delta_time,
            )
        }
        7 => {
            if here != Dimension::Cairn {
                advance(&mut resources);
            } else {
                let lit = resources.lit[queries::tone_index(Dimension::Cairn)];
                let target = if lit {
                    data::CAIRN_GATE
                } else {
                    data::CAIRN_PEAK
                };
                let pose = resources.digger.position;
                let yaw = resources.digger.yaw;
                drive = walk_pose(pose, yaw, target, delta_time);
                drive.jump = resources.digger.grounded && resources.autoplay.held > 0.9;
                if drive.jump {
                    resources.autoplay.held = 0.0;
                }
            }
        }
        8 => {
            drive = travel(
                &mut resources,
                here,
                Dimension::Hearth,
                data::AUTO_HEARTH,
                delta_time,
            )
        }
        9 => {
            if here != Dimension::Hearth {
                advance(&mut resources);
            } else {
                let lit = resources.lit[queries::tone_index(Dimension::Hearth)];
                let target = if lit {
                    data::HEARTH_GATE
                } else {
                    data::HEARTH_WELL
                };
                let reach = target - resources.villager.position;
                let flat = Vec3::new(reach.x, 0.0, reach.z);
                if flat.norm() > 0.3 {
                    let heading = flat.normalize();
                    drive.wish = Vec2::new(heading.x, -heading.z);
                }
            }
        }
        10 => {
            if queries::within_dials(&resources.walker) {
                advance(&mut resources);
            } else {
                drive = walk_to(&resources, data::AUTO_DIALS, delta_time);
            }
        }
        11 => {
            if resources.arch.open {
                advance(&mut resources);
            } else if resources.autoplay.held >= data::AUTO_TURN_SECONDS {
                resources.autoplay.held = 0.0;
                drive.dial = wrong_dial(&resources);
            }
        }
        12 => {
            if queries::at_spot(resources.walker.position, Vec3::zeros(), 1.2) {
                advance(&mut resources);
            } else {
                drive = walk_to(&resources, Vec3::zeros(), delta_time);
            }
        }
        _ => {}
    }

    resources.drive = drive;
}

fn travel(
    resources: &mut FourfoldResources,
    here: Dimension,
    wanted: Dimension,
    arch: Vec3,
    delta_time: f32,
) -> Drive {
    if here == wanted {
        resources.autoplay.moves = 0;
        advance(resources);
        return Drive::default();
    }
    walk_to(resources, arch, delta_time)
}

fn advance(resources: &mut FourfoldResources) {
    resources.autoplay.step += 1;
    resources.autoplay.held = 0.0;
}

fn wrong_dial(resources: &FourfoldResources) -> Option<usize> {
    (0..data::DIAL_COUNT).find(|dial| resources.arch.dials[*dial] != queries::wanted_glyph(*dial))
}

fn next_move(resources: &mut FourfoldResources) -> Vec2 {
    let index = resources.autoplay.moves;
    if index >= data::LOOM_SOLUTION.len() {
        return Vec2::zeros();
    }
    if resources.stepper.rest > 0.0 {
        return Vec2::zeros();
    }
    resources.autoplay.moves += 1;
    let heading = data::LOOM_SOLUTION[index];
    Vec2::new(heading[0] as f32, -heading[1] as f32)
}

fn wants_jump(resources: &mut FourfoldResources) -> bool {
    let index = resources.autoplay.jumps;
    if index >= data::AUTO_JUMPS.len() || !resources.runner.grounded {
        return false;
    }
    if resources.runner.position.x < data::AUTO_JUMPS[index] {
        return false;
    }
    resources.autoplay.jumps += 1;
    true
}

fn walk_to(resources: &FourfoldResources, target: Vec3, delta_time: f32) -> Drive {
    walk_pose(
        resources.walker.position,
        resources.walker.yaw,
        target,
        delta_time,
    )
}

fn walk_pose(position: Vec3, yaw: f32, target: Vec3, delta_time: f32) -> Drive {
    let flat = Vec3::new(target.x - position.x, 0.0, target.z - position.z);
    if flat.norm() < 0.2 {
        return Drive::default();
    }
    let heading = flat.normalize();
    let wanted = (-heading.x).atan2(-heading.z);
    let turn = wrapped(wanted - yaw);
    let step = data::AUTO_TURN_RATE * delta_time;
    let pace = if turn.abs() < data::AUTO_FACING {
        1.0
    } else {
        0.35
    };
    Drive {
        wish: Vec2::new(0.0, pace),
        look: Vec2::new(-turn.clamp(-step, step), 0.0),
        jump: false,
        dial: None,
    }
}

fn wrapped(angle: f32) -> f32 {
    let turn = std::f32::consts::TAU;
    let held = (angle + std::f32::consts::PI).rem_euclid(turn);
    held - std::f32::consts::PI
}
