use crate::data::edit::OPERATORS;
use crate::data::{
    Cardinality, Expression, Field, Port, Program, Statement, System, UnaryOperator,
};

pub fn program(held: &Program) -> String {
    let mut lines = Vec::new();
    for (name, source, root, rows) in &held.assets {
        if root.is_empty() {
            lines.push(format!("assets {name} from {source}"));
        } else {
            lines.push(format!("assets {name} from {source} \"{root}\""));
        }
        for row in rows {
            lines.push(format!("   {} {} \"{}\"", row.label, row.kind, row.file));
        }
        lines.push(String::new());
    }
    for name in &held.components {
        lines.push(format!("component {name}"));
    }
    if !held.components.is_empty() {
        lines.push(String::new());
    }
    for (name, fields) in &held.configs {
        lines.push(format!("config {name}"));
        record(&mut lines, fields);
        lines.push(String::new());
    }
    for (name, fields) in &held.resources {
        lines.push(format!("resource {name}"));
        record(&mut lines, fields);
        lines.push(String::new());
    }
    for name in &held.events {
        lines.push(format!("event {name}"));
    }
    if !held.events.is_empty() {
        lines.push(String::new());
    }
    for (name, parameters, body) in &held.queries {
        lines.push(format!(
            "query {name}({}) = {}",
            parameters.join(", "),
            value(body)
        ));
    }
    if !held.queries.is_empty() {
        lines.push(String::new());
    }
    for system in &held.systems {
        lines.extend(sentence(system));
        lines.push(String::new());
    }
    for (earlier, later) in &held.orders {
        lines.push(format!("order {earlier} before {later}"));
    }
    lines.join("\n")
}

fn record(lines: &mut Vec<String>, fields: &[Field]) {
    let mut open: Vec<String> = Vec::new();
    for field in fields {
        let mut parts: Vec<&str> = field.name.split('.').collect();
        let name = parts.pop().unwrap_or_default();
        while open.len() > parts.len() || (!open.is_empty() && open[..] != parts[..open.len()]) {
            open.pop();
        }
        for step in parts.iter().skip(open.len()) {
            lines.push(format!("{}{step}", "   ".repeat(open.len() + 1)));
            open.push((*step).to_string());
        }
        let lead = "   ".repeat(open.len() + 1);
        match &field.default {
            Some(held) => lines.push(format!("{lead}{name} = {}", value(held))),
            None => lines.push(format!("{lead}{name}")),
        }
    }
}

pub fn sentence(system: &System) -> Vec<String> {
    let mut lines = vec![format!("system {}", system.name)];
    for port in &system.ports {
        lines.push(format!("   {}", gate(port)));
    }
    if !system.ports.is_empty() && !system.body.is_empty() {
        lines.push(String::new());
    }
    block(&mut lines, &system.body, 1);
    lines
}

fn gate(port: &Port) -> String {
    match port {
        Port::Reads(names) => format!("in {}", names.join(", ")),
        Port::Writes(names) => format!("out {}", names.join(", ")),
        Port::Enable(condition) => format!("enable {}", value(condition)),
        Port::Once => "once".to_string(),
        Port::Creates => "creates entities".to_string(),
        Port::Removes => "removes entities".to_string(),
        Port::Phase(name) => format!("phase {name}"),
        Port::Pure => "pure".to_string(),
        Port::Needs(names) => format!("needs {}", names.join(", ")),
        Port::Provides(names) => format!("provides {}", names.join(", ")),
        Port::Select(selection) => {
            let lead = match selection.cardinality {
                Cardinality::Every => "over",
                Cardinality::One => "one",
                Cardinality::Set => "uses",
            };
            let loose = if selection.loose { "any order " } else { "" };
            let mut held = format!("{loose}{lead} {}: ", selection.label);
            let columns: Vec<String> = selection
                .columns
                .iter()
                .map(|column| {
                    let way = if column.written { "out" } else { "in" };
                    format!("{way} {}", column.name)
                })
                .collect();
            held.push_str(&columns.join(", "));
            if !selection.requires.is_empty() {
                held.push_str(&format!(" with {}", selection.requires.join(", ")));
            }
            if !selection.excludes.is_empty() {
                held.push_str(&format!(" without {}", selection.excludes.join(", ")));
            }
            held
        }
    }
}

fn block(lines: &mut Vec<String>, body: &[Statement], depth: usize) {
    let lead = "   ".repeat(depth);
    for statement in body {
        match statement {
            Statement::Let { name, value: held } => {
                lines.push(format!("{lead}let {name} := {}", value(held)))
            }
            Statement::Assign { place, value: held } => {
                lines.push(format!("{lead}{} := {}", spot(place), value(held)))
            }
            Statement::Add { place, value: held } => {
                lines.push(format!("{lead}add {} to {}", value(held), spot(place)))
            }
            Statement::Increase { place, value: held } => {
                lines.push(format!("{lead}{} += {}", spot(place), value(held)))
            }
            Statement::Spawn { bindings } => {
                let parts: Vec<String> = bindings
                    .iter()
                    .map(|(name, held)| format!("{name} {}", value(held)))
                    .collect();
                lines.push(format!("{lead}spawn {}", parts.join(", ")));
            }
            Statement::Remove { subject } => lines.push(format!("{lead}remove {}", value(subject))),
            Statement::Keep { name } => lines.push(format!("{lead}keep {name}")),
            Statement::Recall { name } => lines.push(format!("{lead}recall {name}")),
            Statement::Send { name, value: held } => match held {
                Expression::Truth(true) => lines.push(format!("{lead}send {name}")),
                other => lines.push(format!("{lead}send {name} {}", value(other))),
            },
            Statement::Emit { name, value: held } => match held {
                Expression::Truth(true) => lines.push(format!("{lead}emit {name}")),
                _ => lines.push(format!("{lead}emit {name} {}", value(held))),
            },
            Statement::When {
                condition,
                then,
                otherwise,
            } => {
                lines.push(format!("{lead}when {}", value(condition)));
                block(lines, then, depth + 1);
                if !otherwise.is_empty() {
                    lines.push(format!("{lead}else"));
                    block(lines, otherwise, depth + 1);
                }
            }
            Statement::ForEach { name, over, body } => {
                lines.push(format!("{lead}for each {name} in {}", value(over)));
                block(lines, body, depth + 1);
            }
            Statement::Break => lines.push(format!("{lead}break")),
            Statement::Stop => lines.push(format!("{lead}stop")),
        }
    }
}

pub fn spot(place: &crate::data::Place) -> String {
    let mut held = place.name.clone();
    if let Some(subject) = &place.subject {
        held.push_str(&format!("({})", value(subject)));
    }
    if let Some(field) = &place.field {
        held.push('.');
        held.push_str(field);
    }
    if let Some(index) = &place.index {
        held.push_str(&format!("[{}]", value(index)));
    }
    held
}

pub fn value(held: &Expression) -> String {
    match held {
        Expression::Number(one) => number(*one),
        Expression::Text(one) => format!("\"{one}\""),
        Expression::Truth(one) => one.to_string(),
        Expression::Name(one) => one.clone(),
        Expression::Field { of, name } => format!("{}.{name}", value(of)),
        Expression::Index { of, at } => format!("{}[{}]", value(of), value(at)),
        Expression::Call { name, arguments } => {
            let parts: Vec<String> = arguments.iter().map(value).collect();
            format!("{name}({})", parts.join(", "))
        }
        Expression::Unary {
            operator,
            value: one,
        } => match operator {
            UnaryOperator::Negate => format!("0 - {}", value(one)),
            UnaryOperator::Not => format!("not {}", value(one)),
        },
        Expression::Binary {
            operator,
            left,
            right,
        } => {
            let mark = OPERATORS
                .iter()
                .find(|(_, held)| held == operator)
                .map(|(mark, _)| *mark)
                .unwrap_or("+");
            format!("({} {mark} {})", value(left), value(right))
        }
        Expression::Tuple(parts) => {
            let parts: Vec<String> = parts.iter().map(value).collect();
            format!("({})", parts.join(", "))
        }
        Expression::List(parts) => {
            let parts: Vec<String> = parts.iter().map(value).collect();
            format!("[{}]", parts.join(", "))
        }
        Expression::Record(parts) => {
            let parts: Vec<String> = parts
                .iter()
                .map(|(name, held)| format!("{name} {}", value(held)))
                .collect();
            format!("{{ {} }}", parts.join(", "))
        }
        Expression::IsSome(one) => format!("{} is some", value(one)),
    }
}

pub fn number(one: f32) -> String {
    if one == one.trunc() && one.abs() < 1.0e9 {
        return format!("{}", one as i64);
    }
    let held = format!("{one:.4}");
    held.trim_end_matches('0').trim_end_matches('.').to_string()
}
