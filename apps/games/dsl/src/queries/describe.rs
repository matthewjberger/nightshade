use crate::data::{BinaryOperator, Expression, Place, Statement, UnaryOperator};

pub fn body(statements: &[Statement]) -> Vec<(u8, String)> {
    let mut lines = Vec::new();
    walk(statements, 0, &mut lines);
    lines
}

fn walk(statements: &[Statement], depth: u8, lines: &mut Vec<(u8, String)>) {
    for statement in statements {
        match statement {
            Statement::Let { name, value } => {
                lines.push((depth, format!("let {name} be {}", phrase(value))))
            }
            Statement::Assign { place, value } => {
                lines.push((depth, format!("set {} to {}", target(place), phrase(value))))
            }
            Statement::Add { place, value } => {
                lines.push((depth, format!("add {} to {}", phrase(value), target(place))))
            }
            Statement::Increase { place, value } => lines.push((
                depth,
                format!("increase {} by {}", target(place), phrase(value)),
            )),
            Statement::Spawn { bindings } => {
                let held: Vec<String> = bindings
                    .iter()
                    .map(|(name, value)| format!("{name} {}", phrase(value)))
                    .collect();
                lines.push((depth, format!("make a thing with {}", held.join(", "))));
            }
            Statement::Remove { subject } => {
                lines.push((depth, format!("remove {}", phrase(subject))))
            }
            Statement::Keep { name } => lines.push((depth, format!("write {name} down"))),
            Statement::Recall { name } => lines.push((depth, format!("read {name} back"))),
            Statement::Send { name, value } => {
                lines.push((depth, format!("ask {name} with {}", phrase(value))))
            }
            Statement::Emit { name, value } => {
                let payload = phrase(value);
                if payload == "true" {
                    lines.push((depth, format!("announce {name}")));
                } else {
                    lines.push((depth, format!("announce {name} carrying {payload}")));
                }
            }
            Statement::When {
                condition,
                then,
                otherwise,
            } => {
                lines.push((depth, format!("when {}", phrase(condition))));
                walk(then, depth + 1, lines);
                if !otherwise.is_empty() {
                    lines.push((depth, "otherwise".to_string()));
                    walk(otherwise, depth + 1, lines);
                }
            }
            Statement::ForEach { name, over, body } => {
                lines.push((depth, format!("for each {name} in {}", phrase(over))));
                walk(body, depth + 1, lines);
            }
            Statement::Break => lines.push((depth, "leave the loop".to_string())),
            Statement::Stop => lines.push((depth, "stop this module".to_string())),
        }
    }
}

fn target(place: &Place) -> String {
    let mut held = place.name.clone();
    if let Some(subject) = &place.subject {
        held.push_str(&format!(" of {}", phrase(subject)));
    }
    if let Some(field) = &place.field {
        held.push('.');
        held.push_str(field);
    }
    if let Some(index) = &place.index {
        held.push_str(&format!(" at {}", phrase(index)));
    }
    held
}

fn phrase(expression: &Expression) -> String {
    match expression {
        Expression::Number(value) => {
            if (value - value.round()).abs() < 1.0e-4 {
                format!("{}", value.round() as i64)
            } else {
                format!("{value}")
            }
        }
        Expression::Text(value) => format!("\"{value}\""),
        Expression::Truth(value) => value.to_string(),
        Expression::Name(name) => name.clone(),
        Expression::Field { of, name } => format!("{}.{name}", phrase(of)),
        Expression::Index { of, at } => format!("{} at {}", phrase(of), phrase(at)),
        Expression::Call { name, arguments } => {
            let held: Vec<String> = arguments.iter().map(phrase).collect();
            format!("{name}({})", held.join(", "))
        }
        Expression::Unary { operator, value } => match operator {
            UnaryOperator::Negate => format!("minus {}", phrase(value)),
            UnaryOperator::Not => format!("not {}", phrase(value)),
        },
        Expression::Binary {
            operator,
            left,
            right,
        } => format!("{} {} {}", phrase(left), word(*operator), phrase(right)),
        Expression::Tuple(parts) => {
            let held: Vec<String> = parts.iter().map(phrase).collect();
            format!("({})", held.join(", "))
        }
        Expression::List(items) => {
            let held: Vec<String> = items.iter().map(phrase).collect();
            format!("[{}]", held.join(", "))
        }
        Expression::Record(fields) => {
            let held: Vec<String> = fields
                .iter()
                .map(|(name, value)| format!("{name} {}", phrase(value)))
                .collect();
            format!("{{{}}}", held.join(", "))
        }
        Expression::IsSome(inner) => format!("{} is there", phrase(inner)),
    }
}

fn word(operator: BinaryOperator) -> &'static str {
    match operator {
        BinaryOperator::Add => "+",
        BinaryOperator::Subtract => "-",
        BinaryOperator::Multiply => "*",
        BinaryOperator::Divide => "/",
        BinaryOperator::Less => "below",
        BinaryOperator::LessOrEqual => "at most",
        BinaryOperator::Greater => "above",
        BinaryOperator::GreaterOrEqual => "at least",
        BinaryOperator::Equal => "is",
        BinaryOperator::NotEqual => "is not",
        BinaryOperator::And => "and",
        BinaryOperator::Or => "or",
        BinaryOperator::Range => "up to",
    }
}
