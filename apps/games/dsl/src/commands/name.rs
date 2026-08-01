use crate::data::{Expression, Field, Place, Port, Program, Selection, Statement, System};

pub const MARK: char = '/';

pub fn qualify(one: &mut Program, module: &str, visible: &[(String, Vec<String>)]) {
    let held = crate::queries::plan::owned(one);
    let mine = Pen {
        module: module.to_string(),
        held,
        visible: visible.to_vec(),
    };

    for name in &mut one.shows {
        *name = mine.own(name);
    }
    for name in &mut one.components {
        *name = mine.own(name);
    }
    for name in &mut one.events {
        *name = mine.own(name);
    }
    for name in &mut one.commands {
        *name = mine.own(name);
    }
    for name in &mut one.pulses {
        *name = mine.own(name);
    }
    for (name, _) in &mut one.shapes {
        *name = mine.own(name);
    }
    for (name, _) in &mut one.payloads {
        *name = mine.own(name);
    }
    for (name, fields) in one.resources.iter_mut().chain(one.configs.iter_mut()) {
        *name = mine.own(name);
        mine.defaults(fields);
    }
    for (name, _, _, rows) in &mut one.assets {
        *name = mine.own(name);
        let _ = rows;
    }
    for (name, parameters, body) in &mut one.queries {
        *name = mine.own(name);
        let mut scope = parameters.clone();
        mine.value(body, &mut scope);
    }
    for (earlier, later) in &mut one.orders {
        *earlier = mine.title(earlier);
        *later = mine.title(later);
    }
    for system in &mut one.systems {
        mine.unit(system);
    }
}

struct Pen {
    module: String,
    held: Vec<String>,
    visible: Vec<(String, Vec<String>)>,
}

impl Pen {
    fn own(&self, name: &str) -> String {
        format!("{}{MARK}{name}", self.module)
    }

    fn title(&self, name: &str) -> String {
        format!("{}{MARK}{name}", self.module)
    }

    fn mine(&self, name: &str) -> bool {
        self.held.iter().any(|one| one == name)
    }

    fn seen(&self, name: &str) -> bool {
        self.visible.iter().any(|(one, _)| one == name)
    }

    fn lent(&self, name: &str) -> Option<String> {
        self.visible
            .iter()
            .find(|(_, names)| names.iter().any(|one| one == name))
            .map(|(module, _)| format!("{module}{MARK}{name}"))
    }

    fn resolve(&self, name: &str) -> Option<String> {
        if self.mine(name) {
            return Some(self.own(name));
        }
        self.lent(name)
    }

    fn dotted(&self, name: &str) -> String {
        let Some((head, rest)) = name.split_once('.') else {
            return self.resolve(name).unwrap_or_else(|| name.to_string());
        };
        if self.seen(head) {
            return format!("{head}{MARK}{rest}");
        }
        match self.resolve(head) {
            Some(held) => format!("{held}.{rest}"),
            None => name.to_string(),
        }
    }

    fn defaults(&self, fields: &mut [Field]) {
        for field in fields {
            if let Some(value) = &mut field.default {
                let mut scope = Vec::new();
                self.value(value, &mut scope);
            }
        }
    }

    fn unit(&self, system: &mut System) {
        system.name = self.title(&system.name);
        let mut scope: Vec<String> = Vec::new();
        for port in &mut system.ports {
            match port {
                Port::Reads(names) | Port::Writes(names) => {
                    for name in names.iter_mut() {
                        *name = self.dotted(name);
                    }
                }
                Port::Select(selection) => {
                    scope.push(selection.label.clone());
                    self.chosen(selection, &scope);
                }
                Port::Enable(condition) => {
                    let mut bare = scope.clone();
                    self.value(condition, &mut bare);
                }
                _ => {}
            }
        }
        let mut body = scope.clone();
        self.block(&mut system.body, &mut body);
    }

    fn chosen(&self, selection: &mut Selection, scope: &[String]) {
        for column in &mut selection.columns {
            column.name = self.dotted(&column.name);
            if let Some(start) = &mut column.start {
                let mut bare = scope.to_vec();
                self.value(start, &mut bare);
            }
        }
        for name in selection
            .requires
            .iter_mut()
            .chain(selection.excludes.iter_mut())
        {
            *name = self.dotted(name);
        }
    }

    fn block(&self, body: &mut [Statement], scope: &mut Vec<String>) {
        for statement in body {
            match statement {
                Statement::Let { name, value } => {
                    self.value(value, scope);
                    scope.push(name.clone());
                }
                Statement::Assign { place, value }
                | Statement::Add { place, value }
                | Statement::Increase { place, value } => {
                    self.value(value, scope);
                    self.spot(place, scope);
                }
                Statement::Spawn { bindings } => {
                    for (name, value) in bindings.iter_mut() {
                        *name = self.dotted(name);
                        self.value(value, scope);
                    }
                }
                Statement::Remove { subject } => self.value(subject, scope),
                Statement::Emit { name, value } | Statement::Send { name, value } => {
                    *name = self.dotted(name);
                    self.value(value, scope);
                }
                Statement::When {
                    condition,
                    then,
                    otherwise,
                } => {
                    self.value(condition, scope);
                    let mut inner = scope.clone();
                    self.block(then, &mut inner);
                    let mut other = scope.clone();
                    self.block(otherwise, &mut other);
                }
                Statement::ForEach { name, over, body } => {
                    self.value(over, scope);
                    let mut inner = scope.clone();
                    inner.push(name.clone());
                    self.block(body, &mut inner);
                }
                Statement::Keep { name } | Statement::Recall { name } => *name = self.dotted(name),
                Statement::Break | Statement::Stop => {}
            }
        }
    }

    fn spot(&self, place: &mut Place, scope: &mut Vec<String>) {
        if let Some(subject) = &mut place.subject {
            self.value(subject, scope);
        }
        if let Some(at) = &mut place.index {
            self.value(at, scope);
        }
        if scope.iter().any(|one| one == &place.name) {
            return;
        }
        place.name = self.dotted(&place.name);
    }

    fn value(&self, expression: &mut Expression, scope: &mut Vec<String>) {
        match expression {
            Expression::Name(name) => {
                if scope.iter().any(|one| one == name) {
                    return;
                }
                if let Some(held) = self.resolve(name) {
                    *name = held;
                }
            }
            Expression::Field { of, name } => {
                if let Expression::Name(head) = of.as_ref()
                    && !scope.iter().any(|one| one == head)
                    && self.seen(head)
                {
                    *expression = Expression::Name(format!("{head}{MARK}{name}"));
                    return;
                }
                self.value(of, scope);
            }
            Expression::Index { of, at } => {
                self.value(of, scope);
                self.value(at, scope);
            }
            Expression::Call { name, arguments } => {
                if let Some(held) = self.resolve(name) {
                    *name = held;
                }
                for argument in arguments {
                    self.value(argument, scope);
                }
            }
            Expression::Unary { value, .. } => self.value(value, scope),
            Expression::Binary { left, right, .. } => {
                self.value(left, scope);
                self.value(right, scope);
            }
            Expression::Tuple(parts) | Expression::List(parts) => {
                for part in parts {
                    self.value(part, scope);
                }
            }
            Expression::Record(fields) => {
                for (_, value) in fields {
                    self.value(value, scope);
                }
            }
            Expression::IsSome(of) => self.value(of, scope),
            Expression::Number(_) | Expression::Text(_) | Expression::Truth(_) => {}
        }
    }
}
