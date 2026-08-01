use crate::resources::{Ground, HeightAnswer, SolidAnswer};

pub fn answer_height_with(ground: &mut Ground, answer: HeightAnswer) {
    ground.height_at = Some(answer);
}

pub fn answer_solid_with(ground: &mut Ground, answer: SolidAnswer) {
    ground.solid_at = Some(answer);
}
