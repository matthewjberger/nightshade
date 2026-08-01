use crate::contract::{Answer, NearAnswers};
use nalgebra_glm::Vec3;
use nightshade_ecs::Entity;

pub fn answer_of(answers: &NearAnswers, tag: u32) -> Option<Answer> {
    answers
        .answers
        .iter()
        .find(|answer| answer.tag == tag)
        .copied()
}

pub fn nearest_of(answers: &NearAnswers, tag: u32) -> Option<(Entity, f32)> {
    let answer = answer_of(answers, tag)?;
    answer.nearest.map(|entity| (entity, answer.reach))
}

pub fn counted_near(answers: &NearAnswers, tag: u32) -> u32 {
    answer_of(answers, tag)
        .map(|answer| answer.count)
        .unwrap_or(0)
}

pub fn push_of(answers: &NearAnswers, entity: Entity) -> Vec3 {
    answers
        .pushes
        .iter()
        .find(|(held, _)| *held == entity)
        .map(|(_, push)| *push)
        .unwrap_or_else(|| Vec3::new(0.0, 0.0, 0.0))
}
