use crate::data::{Expression, Field, Program};

pub fn block_name(program: &Program, block: usize) -> String {
    if block < program.configs.len() {
        program.configs[block].0.clone()
    } else {
        program
            .resources
            .get(block - program.configs.len())
            .map(|(name, _)| name.clone())
            .unwrap_or_default()
    }
}

pub fn count(program: &Program) -> usize {
    program.configs.len() + program.resources.len()
}

pub fn fields(program: &Program, block: usize) -> &[Field] {
    if block < program.configs.len() {
        &program.configs[block].1
    } else {
        match program.resources.get(block - program.configs.len()) {
            Some((_, held)) => held,
            None => &[],
        }
    }
}

pub fn parts(value: &Expression) -> usize {
    match value {
        Expression::Field { .. } | Expression::Unary { .. } | Expression::IsSome(_) => 1,
        Expression::Index { .. } | Expression::Binary { .. } => 2,
        Expression::Call { arguments, .. }
        | Expression::Tuple(arguments)
        | Expression::List(arguments) => arguments.len(),
        Expression::Record(held) => held.len(),
        _ => 0,
    }
}
