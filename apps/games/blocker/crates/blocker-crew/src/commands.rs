use crate::components::{Haunt, Mark, Marked, Persona, Post, Rest, Speed, Target};
use crate::data::marks::{MARK_HEIGHT, MARK_YELLOW};
use crate::queries::mark_properties;
use blocker_figure::commands::{dress_the_body, plate_the_body};
use blocker_figure::components::{Heading, Home};
use blocker_figure::queries::plate_properties;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::ecs::Entity;
use nightshade::prelude::*;
use nightshade_scene::prelude::{GlobalTransform, LocalTransform, Name};
use nightshade_text::prelude::{TextState, spawn_3d_billboard_text_with_properties};

pub fn stand_a_body(
    commands: &mut Commands,
    name: &str,
    place: Vec3,
    look: usize,
    fit: f32,
    pace: f32,
) -> Entity {
    let spot = Vec2::new(place.x, place.z);
    let body = commands.spawn((
        Name(name.to_string()),
        LocalTransform::from_translation(place),
        GlobalTransform::default(),
        Home(spot),
        Target(spot),
        Heading(0.0),
    ));
    dress_the_body(commands, body, look, fit);
    commands.set(body, Speed(pace));
    body
}

pub fn stand_a_person(
    text_state: &mut TextState,
    commands: &mut Commands,
    place: Vec3,
    look: usize,
    persona: usize,
    pace: f32,
    fit: f32,
) -> Entity {
    let folk = &crate::data::people::FOLK[persona];
    let body = stand_a_body(commands, folk.name, place, look, fit, pace);
    let spot = Vec2::new(place.x, place.z);
    commands.set(body, Haunt(spot));
    commands.set(body, Post(spot));
    commands.set(body, Rest(0.0));
    commands.set(body, Persona(persona));
    plate_the_body(
        text_state,
        commands,
        body,
        &format!("<{}>\n{}", folk.role, folk.name),
        place,
        plate_properties(),
    );
    body
}

pub fn mark_the_giver(
    text_state: &mut TextState,
    commands: &mut Commands,
    body: Entity,
    place: Vec3,
) {
    let mark = spawn_3d_billboard_text_with_properties(
        text_state,
        commands,
        "!",
        place + Vec3::new(0.0, MARK_HEIGHT, 0.0),
        mark_properties(MARK_YELLOW),
    );
    commands.set(body, Mark(mark));
    commands.set(body, Marked::default());
}
