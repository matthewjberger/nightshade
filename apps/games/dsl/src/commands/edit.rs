use crate::data::edit::{ELSE, Home, Spot};
use crate::data::{Expression, Field, Port, Program, Selection, Statement, System};

fn fields_mut(program: &mut Program, block: usize) -> Option<&mut Vec<Field>> {
    let held = program.configs.len();
    if block < held {
        return program.configs.get_mut(block).map(|(_, fields)| fields);
    }
    program
        .resources
        .get_mut(block - held)
        .map(|(_, fields)| fields)
}

pub fn body_at<'a>(body: &'a mut Vec<Statement>, trail: &[u16]) -> Option<&'a mut Vec<Statement>> {
    let mut held = body;
    for step in trail {
        let at = (step & !ELSE) as usize;
        let other = step & ELSE != 0;
        held = match held.get_mut(at)? {
            Statement::When {
                then, otherwise, ..
            } => {
                if other {
                    otherwise
                } else {
                    then
                }
            }
            Statement::ForEach { body, .. } => body,
            _ => return None,
        };
    }
    Some(held)
}

pub fn statement_at<'a>(body: &'a mut Vec<Statement>, trail: &[u16]) -> Option<&'a mut Statement> {
    let (last, above) = trail.split_last()?;
    let held = body_at(body, above)?;
    held.get_mut((last & !ELSE) as usize)
}

fn slot_of(statement: &mut Statement, slot: u16) -> Option<&mut Expression> {
    match statement {
        Statement::Let { value, .. } => (slot == 0).then_some(value),
        Statement::Assign { place, value }
        | Statement::Add { place, value }
        | Statement::Increase { place, value } => match slot {
            0 => Some(value),
            1 => place.subject.as_mut(),
            2 => place.index.as_mut(),
            _ => None,
        },
        Statement::Spawn { bindings } => bindings.get_mut(slot as usize).map(|(_, value)| value),
        Statement::Remove { subject } => (slot == 0).then_some(subject),
        Statement::When { condition, .. } => (slot == 0).then_some(condition),
        Statement::ForEach { over, .. } => (slot == 0).then_some(over),
        Statement::Emit { value, .. } | Statement::Send { value, .. } => {
            (slot == 0).then_some(value)
        }
        Statement::Break | Statement::Stop | Statement::Keep { .. } | Statement::Recall { .. } => {
            None
        }
    }
}

pub fn part_of(value: &mut Expression, at: u16) -> Option<&mut Expression> {
    let step = at as usize;
    match value {
        Expression::Field { of, .. }
        | Expression::Unary { value: of, .. }
        | Expression::IsSome(of) => (at == 0).then(|| of.as_mut()),
        Expression::Index { of, at: index } => match at {
            0 => Some(of.as_mut()),
            1 => Some(index.as_mut()),
            _ => None,
        },
        Expression::Binary { left, right, .. } => match at {
            0 => Some(left.as_mut()),
            1 => Some(right.as_mut()),
            _ => None,
        },
        Expression::Call { arguments, .. }
        | Expression::Tuple(arguments)
        | Expression::List(arguments) => arguments.get_mut(step),
        Expression::Record(held) => held.get_mut(step).map(|(_, value)| value),
        _ => None,
    }
}

pub fn reach<'a>(program: &'a mut Program, spot: &Spot) -> Option<&'a mut Expression> {
    let mut held = match &spot.home {
        Home::Nowhere => None,
        Home::Value { block, field } => {
            let fields = fields_mut(program, *block)?;
            let field = fields.get_mut(*field)?;
            if field.default.is_none() {
                field.default = Some(Expression::Number(0.0));
            }
            field.default.as_mut()
        }
        Home::Gate { system, port } => match program.systems.get_mut(*system)?.ports.get_mut(*port)
        {
            Some(Port::Enable(condition)) => Some(condition),
            _ => None,
        },
        Home::Body {
            system,
            trail,
            slot,
        } => {
            let body = &mut program.systems.get_mut(*system)?.body;
            slot_of(statement_at(body, trail)?, *slot)
        }
    }?;
    for step in &spot.inside {
        held = part_of(held, *step)?;
    }
    Some(held)
}

pub fn set_number(program: &mut Program, spot: &Spot, value: f32) {
    if let Some(held) = reach(program, spot) {
        *held = Expression::Number(value);
    }
}

pub fn set_truth(program: &mut Program, spot: &Spot, value: bool) {
    if let Some(held) = reach(program, spot) {
        *held = Expression::Truth(value);
    }
}

pub fn set_words(program: &mut Program, spot: &Spot, value: &str) {
    if let Some(held) = reach(program, spot) {
        *held = match held {
            Expression::Name(_) => Expression::Name(value.to_string()),
            _ => Expression::Text(value.to_string()),
        };
    }
}

pub fn set_color(program: &mut Program, spot: &Spot, parts: [f32; 3]) {
    if let Some(held) = reach(program, spot) {
        *held = Expression::Tuple(vec![
            Expression::Number(parts[0]),
            Expression::Number(parts[1]),
            Expression::Number(parts[2]),
        ]);
    }
}

pub fn set_kind(program: &mut Program, spot: &Spot, kind: usize) {
    if let Some(held) = reach(program, spot) {
        *held = crate::data::edit::fresh(kind, held);
    }
}

pub fn set_name(program: &mut Program, spot: &Spot, name: &str) {
    let Some(held) = reach(program, spot) else {
        return;
    };
    match name.split_once('.') {
        Some((block, field)) => {
            *held = Expression::Field {
                of: Box::new(Expression::Name(block.to_string())),
                name: field.to_string(),
            }
        }
        None => *held = Expression::Name(name.to_string()),
    }
}

pub fn set_operator(program: &mut Program, spot: &Spot, at: usize) {
    let Some((_, wanted)) = crate::data::edit::OPERATORS.get(at) else {
        return;
    };
    if let Some(Expression::Binary { operator, .. }) = reach(program, spot) {
        *operator = *wanted;
    }
}

pub fn set_call(program: &mut Program, spot: &Spot, wanted: &str) {
    if let Some(Expression::Call { name, .. }) = reach(program, spot) {
        *name = wanted.to_string();
    }
}

pub fn add_part(program: &mut Program, spot: &Spot) {
    let Some(held) = reach(program, spot) else {
        return;
    };
    match held {
        Expression::Call { arguments, .. }
        | Expression::Tuple(arguments)
        | Expression::List(arguments) => arguments.push(Expression::Number(0.0)),
        _ => {}
    }
}

pub fn drop_part(program: &mut Program, spot: &Spot, at: u16) {
    let Some(held) = reach(program, spot) else {
        return;
    };
    let at = at as usize;
    match held {
        Expression::Call { arguments, .. }
        | Expression::Tuple(arguments)
        | Expression::List(arguments)
            if at < arguments.len() && arguments.len() > 1 =>
        {
            arguments.remove(at);
        }
        _ => {}
    }
}

pub fn add_field(program: &mut Program, block: usize, path: &str, name: &str, group: bool) {
    let Some(fields) = fields_mut(program, block) else {
        return;
    };
    let held = name.trim().to_string();
    let held = if held.is_empty() {
        let lead = if group { "group" } else { "value" };
        format!("{lead}{}", fields.len() + 1)
    } else {
        held
    };
    let held = if path.is_empty() {
        held
    } else {
        format!("{path}.{held}")
    };
    let held = if group { format!("{held}.value") } else { held };
    if fields.iter().any(|field| field.name == held) {
        return;
    }
    let at = fields
        .iter()
        .rposition(|field| field.name.starts_with(path) || path.is_empty())
        .map(|at| at + 1)
        .unwrap_or(fields.len());
    fields.insert(
        at.min(fields.len()),
        Field {
            name: held,
            kind: None,
            default: Some(Expression::Number(1.0)),
        },
    );
}

pub fn drop_field(program: &mut Program, block: usize, at: usize) {
    if let Some(fields) = fields_mut(program, block)
        && at < fields.len()
    {
        fields.remove(at);
    }
}

pub fn add_component(program: &mut Program, name: &str) {
    if name.is_empty() || program.components.iter().any(|held| held == name) {
        return;
    }
    program.components.push(name.to_string());
    let module = program.module.clone();
    program.owner.push((name.to_string(), module));
}

pub fn drop_component(program: &mut Program, at: usize) {
    if at < program.components.len() {
        let name = program.components.remove(at);
        program.owner.retain(|(held, _)| held != &name);
    }
}

pub fn add_system(program: &mut Program, name: &str) {
    if name.is_empty() || program.systems.iter().any(|held| held.name == name) {
        return;
    }
    program.systems.push(System {
        name: name.to_string(),
        module: program.module.clone(),
        ports: Vec::new(),
        body: Vec::new(),
    });
}

pub fn drop_system(program: &mut Program, at: usize) {
    if at < program.systems.len() {
        program.systems.remove(at);
    }
}

pub fn add_port(program: &mut Program, system: usize, kind: usize) {
    let Some(held) = program.systems.get_mut(system) else {
        return;
    };
    let label = format!("held{}", held.ports.len());
    let selection = |cardinality| {
        Port::Select(Selection {
            label,
            cardinality,
            loose: false,
            columns: vec![crate::data::Column {
                name: crate::data::TRANSFORM_COLUMN.to_string(),
                written: false,
                start: None,
            }],
            requires: Vec::new(),
            excludes: Vec::new(),
        })
    };
    let port = match kind {
        0 => Port::Reads(vec![crate::data::TIME_RESOURCE.to_string()]),
        1 => Port::Writes(vec![crate::data::WINDOW_RESOURCE.to_string()]),
        2 => selection(crate::data::Cardinality::Every),
        3 => selection(crate::data::Cardinality::One),
        4 => selection(crate::data::Cardinality::Set),
        5 => Port::Enable(Expression::Truth(true)),
        6 => Port::Once,
        7 => Port::Creates,
        8 => Port::Removes,
        9 => Port::Phase("update".to_string()),
        10 => Port::Needs(Vec::new()),
        11 => Port::Provides(Vec::new()),
        _ => Port::Pure,
    };
    held.ports.push(port);
}

pub fn drop_port(program: &mut Program, system: usize, at: usize) {
    if let Some(held) = program.systems.get_mut(system)
        && at < held.ports.len()
    {
        held.ports.remove(at);
    }
}

pub fn set_port_name(program: &mut Program, system: usize, port: usize, name: &str) {
    let Some(held) = program.systems.get_mut(system) else {
        return;
    };
    match held.ports.get_mut(port) {
        Some(Port::Reads(names)) | Some(Port::Writes(names)) => {
            names.clear();
            names.push(name.to_string());
        }
        Some(Port::Select(selection)) => {
            selection.columns.clear();
            selection.columns.push(crate::data::Column {
                name: name.to_string(),
                written: false,
                start: None,
            });
        }
        _ => {}
    }
}

pub fn add_statement(program: &mut Program, system: usize, trail: &[u16], form: usize) {
    let Some(held) = program.systems.get_mut(system) else {
        return;
    };
    let statement = crate::data::edit::open(form);
    match trail.split_last() {
        None => held.body.push(statement),
        Some((last, above)) => {
            let Some(body) = body_at(&mut held.body, above) else {
                return;
            };
            let at = ((last & !ELSE) as usize + 1).min(body.len());
            body.insert(at, statement);
        }
    }
}

pub fn add_inside(program: &mut Program, system: usize, trail: &[u16], form: usize) {
    let Some(held) = program.systems.get_mut(system) else {
        return;
    };
    if let Some(body) = body_at(&mut held.body, trail) {
        body.push(crate::data::edit::open(form));
    }
}

pub fn drop_statement(program: &mut Program, system: usize, trail: &[u16]) {
    let Some(held) = program.systems.get_mut(system) else {
        return;
    };
    let Some((last, above)) = trail.split_last() else {
        return;
    };
    let Some(body) = body_at(&mut held.body, above) else {
        return;
    };
    let at = (last & !ELSE) as usize;
    if at < body.len() {
        body.remove(at);
    }
}

pub fn lift_statement(program: &mut Program, system: usize, trail: &[u16], down: bool) {
    let Some(held) = program.systems.get_mut(system) else {
        return;
    };
    let Some((last, above)) = trail.split_last() else {
        return;
    };
    let Some(body) = body_at(&mut held.body, above) else {
        return;
    };
    let at = (last & !ELSE) as usize;
    let wanted = if down { at + 1 } else { at.wrapping_sub(1) };
    if at < body.len() && wanted < body.len() {
        body.swap(at, wanted);
    }
}

pub fn set_place(program: &mut Program, system: usize, trail: &[u16], name: &str) {
    let Some(held) = program.systems.get_mut(system) else {
        return;
    };
    let Some(statement) = statement_at(&mut held.body, trail) else {
        return;
    };
    let (place, field) = match statement {
        Statement::Assign { place, .. } | Statement::Add { place, .. } => {
            (place, name.split_once('.'))
        }
        Statement::Let { name: held, .. }
        | Statement::ForEach { name: held, .. }
        | Statement::Emit { name: held, .. }
        | Statement::Send { name: held, .. } => {
            *held = name.to_string();
            return;
        }
        _ => return,
    };
    match field {
        Some((block, field)) => {
            place.name = block.to_string();
            place.field = Some(field.to_string());
        }
        None => {
            place.name = name.to_string();
            place.field = None;
        }
    }
}

pub fn set_binding(program: &mut Program, system: usize, trail: &[u16], at: usize, name: &str) {
    let Some(held) = program.systems.get_mut(system) else {
        return;
    };
    if let Some(Statement::Spawn { bindings }) = statement_at(&mut held.body, trail)
        && let Some(binding) = bindings.get_mut(at)
    {
        binding.0 = name.to_string();
    }
}

pub fn add_binding(program: &mut Program, system: usize, trail: &[u16]) {
    let Some(held) = program.systems.get_mut(system) else {
        return;
    };
    if let Some(Statement::Spawn { bindings }) = statement_at(&mut held.body, trail) {
        bindings.push((
            crate::data::COLOR_BINDING.to_string(),
            Expression::Tuple(vec![
                Expression::Number(1.0),
                Expression::Number(1.0),
                Expression::Number(1.0),
            ]),
        ));
    }
}
