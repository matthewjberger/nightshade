use crate::resources::{Cells, InBoundsAnswer, StepCostAnswer, WalkableAnswer};

pub fn answer_walkable_with(cells: &mut Cells, answer: WalkableAnswer) {
    cells.walkable = Some(answer);
}

pub fn answer_step_cost_with(cells: &mut Cells, answer: StepCostAnswer) {
    cells.step_cost = Some(answer);
}

pub fn answer_in_bounds_with(cells: &mut Cells, answer: InBoundsAnswer) {
    cells.in_bounds = Some(answer);
}
