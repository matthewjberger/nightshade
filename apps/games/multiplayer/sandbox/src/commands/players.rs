use crate::components::{
    At, Avatar, Crouch, Facing, Fit, Grounded, JumpHold, Look, Peer, Steering, Track, Velocity,
    Wants, WasAt, WasFacing,
};
use crate::data::CHARACTERS;
use crate::data::wire::PeerId;
use crate::queries::model;
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_scene::prelude::{GlobalTransform, LocalTransform, Name};

type Seat = (
    Name,
    LocalTransform,
    GlobalTransform,
    Peer,
    Look,
    Fit,
    Velocity,
    Facing,
    Crouch,
    Grounded,
    JumpHold,
    Avatar,
);

fn seat(peer: PeerId, character: u8, position: Vec3) -> Seat {
    let name = CHARACTERS[(character as usize) % CHARACTERS.len()];
    (
        Name(format!("player {peer}")),
        LocalTransform {
            translation: position,
            rotation: nalgebra_glm::Quat::identity(),
            scale: Vec3::new(1.0, 1.0, 1.0),
        },
        GlobalTransform::default(),
        Peer(peer),
        Look(model(name)),
        Fit(theme::CHARACTER_SCALE),
        Velocity(Vec3::zeros()),
        Facing(0.0),
        Crouch(0.0),
        Grounded(false),
        JumpHold(0.0),
        Avatar(None),
    )
}

pub fn spawn_driven(
    commands: &mut Commands,
    peer: PeerId,
    character: u8,
    position: Vec3,
) -> Entity {
    let entity = commands.spawn(seat(peer, character, position));
    commands.queue(move |storage, _resources| {
        storage.set(entity, Steering(Vec3::zeros()));
        storage.set(entity, Wants(0));
        storage.set(entity, At(position));
        storage.set(entity, WasAt(position));
        storage.set(entity, WasFacing(0.0));
    });
    entity
}

pub fn spawn_tracked(
    commands: &mut Commands,
    peer: PeerId,
    character: u8,
    position: Vec3,
) -> Entity {
    let (
        name,
        transform,
        global,
        peer,
        look,
        fit,
        velocity,
        facing,
        crouch,
        grounded,
        hold,
        avatar,
    ) = seat(peer, character, position);
    commands.spawn((
        name,
        transform,
        global,
        peer,
        look,
        fit,
        velocity,
        facing,
        crouch,
        grounded,
        hold,
        avatar,
        Track(std::collections::VecDeque::new()),
    ))
}
