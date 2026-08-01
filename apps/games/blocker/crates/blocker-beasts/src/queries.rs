pub fn beast_material(index: usize) -> String {
    format!("blocker_beast_{index}")
}

pub fn beast_snout(index: usize) -> String {
    format!("blocker_snout_{index}")
}

pub fn facing_offset(kind: usize) -> f32 {
    match crate::data::beasts::BEASTS[kind].form {
        crate::data::beasts::Form::Cube => std::f32::consts::PI,
        crate::data::beasts::Form::Figure { .. } => 0.0,
    }
}
