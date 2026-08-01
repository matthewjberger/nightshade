use crate::data::cell::axis_index;
use crate::data::facility;
use crate::data::form::{self, Form};
use crate::data::plan::{Known, LoopSite, Op, Pick, Plan, Routine, Unit};
use crate::data::{self, Expression, Place, Port, Program, Statement, System};
use crate::queries::describe;
use std::collections::HashMap;

pub fn build(program: &Program, turn: Vec<usize>) -> Result<Plan, String> {
    let mut maker = Maker::default();
    maker.gather(program);
    maker.plan.turn = turn;

    for (name, parameters, body) in &program.queries {
        let index = maker.plan.queries.len();
        maker.query_index.insert(name.clone(), index);
        maker.plan.queries.push((Routine::default(), 0));
        let mut frame = Frame::default();
        for parameter in parameters {
            frame.declare(parameter);
        }
        let mut code = Vec::new();
        maker.expression(&mut code, &mut frame, body)?;
        maker.plan.queries[index] = (Routine { code }, frame.count);
    }

    for (block, fields) in program.configs.iter().chain(program.resources.iter()) {
        for field in fields {
            let slot = maker.field_slot(block, &field.name);
            let mut code = Vec::new();
            match &field.default {
                Some(value) => {
                    let mut frame = Frame::default();
                    maker.expression(&mut code, &mut frame, value)?;
                }
                None => code.push(Op::PushNumber(0.0)),
            }
            maker.plan.seeds.push((slot, Routine { code }));
        }
    }
    for (block, _, root, rows) in &program.assets {
        for row in rows {
            let slot = maker.field_slot(block, &row.label);
            let text = maker.text(&data::joined(root, &row.file));
            maker.plan.seeds.push((
                slot,
                Routine {
                    code: vec![Op::PushText(text)],
                },
            ));
        }
    }

    for system in &program.systems {
        let unit = maker.unit(system)?;
        maker.plan.units.push(unit);
    }
    maker.plan.known.actions = program
        .actions
        .iter()
        .filter_map(|(name, key, held)| {
            maker
                .slot_of(data::INPUT_RESOURCE, name)
                .map(|slot| (slot, key.clone(), *held))
        })
        .collect();
    routed(program)?;
    served(&maker.plan)?;
    stage(&mut maker.plan)?;

    for held in [maker.plan.known.clicked, maker.plan.known.picked] {
        if let Some(key) = held
            && !maker.plan.pulses.contains(&key)
        {
            maker.plan.pulses.push(key);
        }
    }
    for name in &program.pulses {
        if let Some(key) = maker.column_keys.get(name) {
            maker.plan.pulses.push(*key);
        }
    }

    maker.settle();
    Ok(maker.plan)
}

#[derive(Default)]
struct Frame {
    names: Vec<(String, u16)>,
    forms: Vec<(u16, Form)>,
    sets: Vec<String>,
    adds: Vec<String>,
    depth: Vec<usize>,
    count: u16,
    payloads: Vec<(u16, String)>,
    loops: Vec<Vec<usize>>,
}

impl Frame {
    fn open(&mut self) {
        self.depth.push(self.names.len());
    }

    fn close(&mut self) {
        if let Some(mark) = self.depth.pop() {
            self.names.truncate(mark);
        }
    }

    fn declare(&mut self, name: &str) -> u16 {
        let slot = self.count;
        self.count += 1;
        self.names.push((name.to_string(), slot));
        slot
    }

    fn note_form(&mut self, slot: u16, form: Option<Form>) {
        self.forms.retain(|(held, _)| *held != slot);
        if let Some(form) = form {
            self.forms.push((slot, form));
        }
    }

    fn form(&self, slot: u16) -> Option<Form> {
        self.forms
            .iter()
            .find(|(held, _)| *held == slot)
            .map(|(_, form)| *form)
    }

    fn find(&self, name: &str) -> Option<u16> {
        self.names
            .iter()
            .rev()
            .find(|(held, _)| held == name)
            .map(|(_, slot)| *slot)
    }

    fn payload(&self, slot: u16) -> Option<&str> {
        self.payloads
            .iter()
            .find(|(held, _)| *held == slot)
            .map(|(_, event)| event.as_str())
    }
}

#[derive(Default)]
struct Maker {
    plan: Plan,
    column_keys: HashMap<String, u16>,
    field_slots: HashMap<(String, String), u16>,
    event_keys: HashMap<String, u16>,
    text_slots: HashMap<String, u32>,
    query_index: HashMap<String, usize>,
    event_fields: HashMap<String, Vec<String>>,
    declared: HashMap<String, Vec<String>>,
    forms: HashMap<String, Form>,
    field_forms: HashMap<(String, String), Form>,
    blocks: Vec<String>,
    variants: Vec<String>,
}

impl Maker {
    fn gather(&mut self, program: &Program) {
        for name in facility::columns() {
            self.column_key(name);
        }
        for name in &program.components {
            self.column_key(name);
        }
        for name in facility::columns() {
            if let Some(held) = facility::form_of(name) {
                self.forms.insert(name.to_string(), held);
            }
        }
        for (name, kind) in &program.shapes {
            if let Some(held) = form::form_of(kind) {
                self.forms.insert(name.clone(), held);
            }
        }
        for block in facility::blocks() {
            for field in block.fields {
                self.field_slot(block.name, field);
            }
        }
        for (name, _, _) in &program.actions {
            self.field_slot(data::INPUT_RESOURCE, name);
        }
        for (block, fields) in program.configs.iter().chain(program.resources.iter()) {
            self.blocks.push(block.clone());
            for field in fields {
                self.field_slot(block, &field.name);
                let held = match &field.kind {
                    Some(kind) => form::form_of(kind),
                    None => field.default.as_ref().and_then(literal_form),
                };
                if let Some(held) = held {
                    self.field_forms
                        .insert((block.clone(), field.name.clone()), held);
                }
            }
        }
        for (block, _, _, rows) in &program.assets {
            self.blocks.push(block.clone());
            for row in rows {
                self.field_slot(block, &row.label);
            }
        }
        for block in facility::blocks() {
            self.blocks.push(block.name.to_string());
        }
        for (name, fields) in &program.payloads {
            self.declared.insert(name.clone(), fields.clone());
            self.event_fields.insert(name.clone(), fields.clone());
        }
        for name in &program.events {
            self.event_key(name);
        }
        self.variants = program.variants.clone();
        for system in &program.systems {
            self.scan(&system.body);
        }
    }

    fn scan(&mut self, body: &[Statement]) {
        for statement in body {
            match statement {
                Statement::Emit {
                    name,
                    value: Expression::Record(fields),
                }
                | Statement::Send {
                    name,
                    value: Expression::Record(fields),
                } => {
                    let held = self.event_fields.entry(name.clone()).or_default();
                    for (field, _) in fields {
                        if !held.iter().any(|one| one == field) {
                            held.push(field.clone());
                        }
                    }
                }
                Statement::When {
                    then, otherwise, ..
                } => {
                    self.scan(then);
                    self.scan(otherwise);
                }
                Statement::ForEach { body, .. } => self.scan(body),
                _ => {}
            }
        }
    }

    fn settle(&mut self) {
        self.plan.flat = self.flat_columns();
        self.plan.holds = self
            .plan
            .columns
            .iter()
            .map(|name| self.forms.get(name).copied() == Some(Form::Number))
            .collect();
        self.plan.known = Known {
            transform: self.column_keys.get(data::TRANSFORM_COLUMN).copied(),
            scale: self.column_keys.get(data::SCALE_BINDING).copied(),
            rotation: self.column_keys.get(data::ROTATION_COLUMN).copied(),
            color: self.column_keys.get(data::COLOR_BINDING).copied(),
            mesh: self.column_keys.get(data::MESH_BINDING).copied(),
            model: self.column_keys.get(data::MODEL_BINDING).copied(),
            parent: self.column_keys.get(data::PARENT_COLUMN).copied(),
            visible: self.column_keys.get(data::VISIBLE_COLUMN).copied(),
            lifetime: self.column_keys.get(data::LIFETIME_COLUMN).copied(),
            label: self.column_keys.get(data::LABEL_COLUMN).copied(),
            size: self.column_keys.get(data::SIZE_COLUMN).copied(),
            anchor: self.column_keys.get(data::ANCHOR_COLUMN).copied(),
            justify: self.column_keys.get(data::JUSTIFY_COLUMN).copied(),
            light: self.column_keys.get(data::LIGHT_COLUMN).copied(),
            strength: self.column_keys.get(data::STRENGTH_COLUMN).copied(),
            reach: self.column_keys.get(data::REACH_COLUMN).copied(),
            sun_angle: self.slot_of(data::SUN_RESOURCE, "angle"),
            sun_color: self.slot_of(data::SUN_RESOURCE, "color"),
            sun_strength: self.slot_of(data::SUN_RESOURCE, "strength"),
            sun_shadows: self.slot_of(data::SUN_RESOURCE, "shadows"),
            sky_show: self.slot_of(data::SKY_RESOURCE, "show"),
            sky_ground: self.slot_of(data::SKY_RESOURCE, "ground"),
            sky_look: self.slot_of(data::SKY_RESOURCE, "look"),
            sky_grid: self.slot_of(data::SKY_RESOURCE, "grid"),
            field: self.column_keys.get(data::FIELD_COLUMN).copied(),
            carve: self.column_keys.get(data::CARVE_COLUMN).copied(),
            blend: self.column_keys.get(data::BLEND_COLUMN).copied(),
            ground: self.column_keys.get(data::GROUND_COLUMN).copied(),
            block: self.column_keys.get(data::BLOCK_COLUMN).copied(),
            dig: self.column_keys.get(data::DIG_COLUMN).copied(),
            land_seed: self.slot_of(data::LAND_RESOURCE, "seed"),
            land_hills: self.slot_of(data::LAND_RESOURCE, "hills"),
            land_shore: self.slot_of(data::LAND_RESOURCE, "shore"),
            land_reach: self.slot_of(data::LAND_RESOURCE, "reach"),
            path: self.column_keys.get(data::PATH_COLUMN).copied(),
            pace: self.column_keys.get(data::PACE_COLUMN).copied(),
            pick: self.column_keys.get(data::PICK_COLUMN).copied(),
            picked: self.column_keys.get(data::PICKED_COLUMN).copied(),
            copies: self.column_keys.get(data::COPIES_COLUMN).copied(),
            spread: self.column_keys.get(data::SPREAD_COLUMN).copied(),
            decal: self.column_keys.get(data::DECAL_BINDING).copied(),
            sprite: self.column_keys.get(data::SPRITE_BINDING).copied(),
            tile: self.column_keys.get(data::TILE_COLUMN).copied(),
            texture: self.column_keys.get(data::TEXTURE_BINDING).copied(),
            repeat: self.column_keys.get(data::REPEAT_COLUMN).copied(),
            cost_steps: self.slot_of(data::COST_RESOURCE, "steps"),
            cost_bytes: self.slot_of(data::COST_RESOURCE, "bytes"),
            cost_commands: self.slot_of(data::COST_RESOURCE, "commands"),
            time_delta: self.slot_of(data::TIME_RESOURCE, "dt"),
            time_uptime: self.slot_of(data::TIME_RESOURCE, "uptime"),
            time_longest: self.slot_of(data::TIME_RESOURCE, "longest"),
            steering: self.slot_of(data::INPUT_RESOURCE, "steering"),
            forward: self.slot_of(data::INPUT_RESOURCE, "forward"),
            serve: self.slot_of(data::INPUT_RESOURCE, "serve"),
            pointer: self.slot_of(data::INPUT_RESOURCE, "pointer"),
            cursor: self.slot_of(data::INPUT_RESOURCE, "cursor"),
            panel: self.column_keys.get(data::PANEL_COLUMN).copied(),
            button: self.column_keys.get(data::BUTTON_COLUMN).copied(),
            clicked: self.column_keys.get(data::CLICKED_COLUMN).copied(),
            body: self.column_keys.get(data::BODY_COLUMN).copied(),
            bounce: self.column_keys.get(data::BOUNCE_COLUMN).copied(),
            ink: self.column_keys.get(data::INK_COLUMN).copied(),
            press: self.slot_of(data::INPUT_RESOURCE, "press"),
            pointer_aim: self.slot_of(data::INPUT_RESOURCE, "aim"),
            title: self.slot_of(data::WINDOW_RESOURCE, "title"),
            close: self.slot_of(data::WINDOW_RESOURCE, "close"),
            rate: self.slot_of(data::WINDOW_RESOURCE, "rate"),
            gpu: self.slot_of(data::WINDOW_RESOURCE, "gpu"),
            picture: self.slot_of(data::WINDOW_RESOURCE, "size"),
            actions: Vec::new(),
            seed: self.slot_of(data::RANDOM_RESOURCE, "seed"),
            flow: self.slot_of(data::FLOW_RESOURCE, "scale"),
            shake_x: self.slot_of(data::SHAKE_RESOURCE, "x"),
            shake_y: self.slot_of(data::SHAKE_RESOURCE, "y"),
            eye: self.slot_of(data::CAMERA_RESOURCE, "eye"),
            aim: self.slot_of(data::CAMERA_RESOURCE, "at"),
            view_height: self
                .field_slots
                .iter()
                .find_map(|((_, field), slot)| (field == data::VIEW_HEIGHT).then_some(*slot)),
            camera_eye: self
                .field_slots
                .iter()
                .find_map(|((_, field), slot)| (field == data::CAMERA_EYE).then_some(*slot)),
        };
    }

    fn slot_of(&self, block: &str, field: &str) -> Option<u16> {
        self.field_slots
            .get(&(block.to_string(), field.to_string()))
            .copied()
    }

    fn column_key(&mut self, name: &str) -> u16 {
        if let Some(key) = self.column_keys.get(name) {
            return *key;
        }
        let key = self.plan.columns.len() as u16;
        self.plan.columns.push(name.to_string());
        self.column_keys.insert(name.to_string(), key);
        key
    }

    fn reached(&self, expression: &Expression) -> Option<(String, String)> {
        let Expression::Field { of, name } = expression else {
            return None;
        };
        let mut parts = vec![name.clone()];
        let mut held = of.as_ref();
        loop {
            match held {
                Expression::Field { of, name } => {
                    parts.push(name.clone());
                    held = of.as_ref();
                }
                Expression::Name(block) if self.is_block(block) => {
                    parts.reverse();
                    let field = parts.join(".");
                    if parts.len() == 1
                        || self
                            .field_slots
                            .contains_key(&(block.clone(), field.clone()))
                    {
                        return Some((block.clone(), field));
                    }
                    return None;
                }
                _ => return None,
            }
        }
    }

    fn field_slot(&mut self, block: &str, field: &str) -> u16 {
        let wanted = (block.to_string(), field.to_string());
        if let Some(slot) = self.field_slots.get(&wanted) {
            return *slot;
        }
        let slot = self.plan.fields.len() as u16;
        self.plan.fields.push(wanted.clone());
        self.field_slots.insert(wanted, slot);
        slot
    }

    fn event_key(&mut self, name: &str) -> u16 {
        if let Some(key) = self.event_keys.get(name) {
            return *key;
        }
        let key = self.plan.events.len() as u16;
        self.plan.events.push(name.to_string());
        self.event_keys.insert(name.to_string(), key);
        key
    }

    fn text(&mut self, value: &str) -> u32 {
        if let Some(slot) = self.text_slots.get(value) {
            return *slot;
        }
        let slot = self.plan.texts.len() as u32;
        self.plan.texts.push(value.to_string());
        self.text_slots.insert(value.to_string(), slot);
        slot
    }

    fn ledger(&mut self, block: &str) -> u16 {
        let mut held: Vec<(u16, String)> = Vec::new();
        for ((name, field), slot) in &self.field_slots {
            if name == block {
                held.push((*slot, format!("{name}.{field}")));
            }
        }
        held.sort_by(|one, two| one.1.cmp(&two.1));
        let site = self.plan.kept.len() as u16;
        self.plan.kept.push(held);
        site
    }

    fn is_block(&self, name: &str) -> bool {
        self.blocks.iter().any(|held| held == name)
    }

    fn unit(&mut self, system: &System) -> Result<Unit, String> {
        let mut frame = Frame::default();
        let mut picks = Vec::new();
        let mut enable = Vec::new();
        let mut once = false;

        for port in &system.ports {
            match port {
                Port::Once => once = true,
                Port::Enable(condition) => {
                    let mut code = Vec::new();
                    let mut bare = Frame::default();
                    self.expression(&mut code, &mut bare, condition)?;
                    enable.push(Routine { code });
                }
                Port::Select(selection) => {
                    let label = frame.declare(&selection.label);
                    if selection.cardinality == data::Cardinality::Set {
                        frame.note_form(label, Some(Form::List));
                    } else {
                        frame.note_form(label, Some(Form::Thing));
                    }
                    let mut need: Vec<u16> = selection
                        .columns
                        .iter()
                        .filter(|column| column.start.is_none())
                        .map(|column| self.column_key(&column.name))
                        .collect();
                    for name in &selection.requires {
                        need.push(self.column_key(name));
                    }
                    let ban = selection
                        .excludes
                        .iter()
                        .map(|name| self.column_key(name))
                        .collect();
                    let mut starts = Vec::new();
                    for column in &selection.columns {
                        if let Some(value) = &column.start {
                            let key = self.column_key(&column.name);
                            let mut code = Vec::new();
                            let mut bare = Frame::default();
                            self.expression(&mut code, &mut bare, value)?;
                            starts.push((key, Routine { code }));
                        }
                    }
                    let roster = self.plan.rosters as u16;
                    self.plan.rosters += 1;
                    picks.push(Pick {
                        roster,
                        label,
                        cardinality: selection.cardinality,
                        loose: selection.loose,
                        need,
                        ban,
                        starts,
                    });
                }
                _ => {}
            }
        }

        let mut code = Vec::new();
        let sites = self.plan.loops.len();
        frame.loops.push(Vec::new());
        self.block(&mut code, &mut frame, &system.body)?;

        let mut reads = Vec::new();
        let mut writes = Vec::new();
        let mut takes = Vec::new();
        let mut gives = Vec::new();
        for port in &system.ports {
            match port {
                Port::Reads(list) => {
                    for name in list {
                        self.sort(name, &mut reads, &mut takes);
                    }
                }
                Port::Writes(list) => {
                    for name in list {
                        self.sort(name, &mut writes, &mut gives);
                    }
                }
                Port::Select(selection) => {
                    for column in &selection.columns {
                        let held = if column.written {
                            &mut writes
                        } else {
                            &mut reads
                        };
                        if !held.contains(&column.name) {
                            held.push(column.name.clone());
                        }
                    }
                    for name in selection.requires.iter().chain(selection.excludes.iter()) {
                        if !reads.contains(name) {
                            reads.push(name.clone());
                        }
                    }
                }
                _ => {}
            }
        }
        let mut asks = Vec::new();
        self.wanted(&system.body, &mut asks);
        let phase = system
            .ports
            .iter()
            .find_map(|port| match port {
                Port::Phase(name) => Some(name.clone()),
                _ => None,
            })
            .unwrap_or_default();
        let pure = system.ports.iter().any(|port| matches!(port, Port::Pure));
        habits(system, pure)?;
        let mut needs = Vec::new();
        let mut serves = Vec::new();
        for port in &system.ports {
            match port {
                Port::Needs(list) => needs.extend(list.iter().cloned()),
                Port::Provides(list) => serves.extend(list.iter().cloned()),
                _ => {}
            }
        }

        let mut watches = Vec::new();
        let mut listens = Vec::new();
        let mut hears = Vec::new();
        if pure {
            for name in reads.iter().chain(writes.iter()) {
                if let Some(key) = self.event_keys.get(name) {
                    hears.push(*key);
                } else if let Some(key) = self.column_keys.get(name) {
                    watches.push(*key);
                }
            }
            for block in takes.iter().chain(gives.iter()) {
                for ((held, _), slot) in &self.field_slots {
                    if held == block {
                        listens.push(*slot);
                    }
                }
            }
        }

        let (code, place) = fuse(&code);
        for site in self.plan.loops[sites..].iter_mut() {
            site.done = place[site.done as usize];
        }
        let wide = broad(&code, &picks, &self.flat_columns());
        Ok(Unit {
            wide,
            name: system.name.clone(),
            module: system.module.clone(),
            phase,
            pure,
            watches,
            listens,
            hears,
            once,
            picks,
            enable,
            body: Routine { code },
            locals: frame.count,
            reads,
            writes,
            takes,
            gives,
            asks,
            needs,
            serves,
            sets: frame.sets.clone(),
            adds: frame.adds.clone(),
            lines: describe::body(&system.body),
        })
    }

    fn block(
        &mut self,
        code: &mut Vec<Op>,
        frame: &mut Frame,
        body: &[Statement],
    ) -> Result<(), String> {
        for statement in body {
            self.statement(code, frame, statement)?;
        }
        Ok(())
    }

    fn statement(
        &mut self,
        code: &mut Vec<Op>,
        frame: &mut Frame,
        statement: &Statement,
    ) -> Result<(), String> {
        match statement {
            Statement::Let { name, value } => {
                let held = self.shape_of(frame, value);
                self.expression(code, frame, value)?;
                let slot = frame.declare(name);
                frame.note_form(slot, held);
                code.push(Op::PopLocal(slot));
            }
            Statement::Assign { place, value } => {
                self.fits(frame, place, value)?;
                let local = place.subject.is_none()
                    && place.field.is_none()
                    && frame.find(&place.name).is_some();
                match (&place.index, local) {
                    (Some(at), false) => {
                        self.expression(code, frame, at)?;
                        self.expression(code, frame, value)?;
                        self.poke_place(code, frame, place)?;
                    }
                    (Some(at), true) => {
                        self.read_place(code, frame, place)?;
                        self.expression(code, frame, at)?;
                        self.expression(code, frame, value)?;
                        code.push(Op::SetIndex);
                        self.write_place(code, frame, place)?;
                    }
                    (None, _) => {
                        self.expression(code, frame, value)?;
                        self.write_place(code, frame, place)?;
                    }
                }
            }
            Statement::Add { place, value } => {
                self.read_place(code, frame, place)?;
                self.expression(code, frame, value)?;
                code.push(Op::Append);
                self.write_place(code, frame, place)?;
            }
            Statement::Increase { place, value } => {
                self.fits(frame, place, value)?;
                let local = place.subject.is_none()
                    && place.field.is_none()
                    && frame.find(&place.name).is_some();
                if local {
                    self.read_place(code, frame, place)?;
                    self.expression(code, frame, value)?;
                    code.push(Op::Binary(data::BinaryOperator::Add));
                    self.write_place(code, frame, place)?;
                } else {
                    self.expression(code, frame, value)?;
                    self.add_place(code, frame, place)?;
                }
            }
            Statement::Spawn { bindings } => {
                let mut keys = Vec::new();
                for (name, value) in bindings {
                    self.matches(frame, name, self.forms.get(name).copied(), value)?;
                    keys.push(self.column_key(name));
                    self.expression(code, frame, value)?;
                }
                let site = self.plan.sites.len() as u16;
                self.plan.sites.push(keys);
                code.push(Op::Spawn(site));
            }
            Statement::Remove { subject } => {
                self.expression(code, frame, subject)?;
                code.push(Op::Remove);
            }
            Statement::Keep { name } | Statement::Recall { name } => {
                let site = self.ledger(name);
                let back = matches!(statement, Statement::Recall { .. });
                code.push(if back {
                    Op::Recall(site)
                } else {
                    Op::Keep(site)
                });
            }
            Statement::Emit { name, value } | Statement::Send { name, value } => {
                self.carried(name, value)?;
                let key = self.event_key(name);
                match value {
                    Expression::Record(fields) => {
                        let order = self.event_fields.get(name).cloned().unwrap_or_default();
                        for wanted in &order {
                            match fields.iter().find(|(field, _)| field == wanted) {
                                Some((_, held)) => self.expression(code, frame, held)?,
                                None => code.push(Op::PushNothing),
                            }
                        }
                        code.push(Op::MakeList(order.len() as u8));
                    }
                    other => self.expression(code, frame, other)?,
                }
                code.push(Op::Emit(key));
            }
            Statement::When {
                condition,
                then,
                otherwise,
            } => {
                self.expression(code, frame, condition)?;
                let branch = code.len();
                code.push(Op::JumpIfFalse(0));
                frame.open();
                self.block(code, frame, then)?;
                frame.close();
                let skip = code.len();
                code.push(Op::Jump(0));
                let middle = code.len() as u32;
                frame.open();
                self.block(code, frame, otherwise)?;
                frame.close();
                let end = code.len() as u32;
                code[branch] = Op::JumpIfFalse(middle);
                code[skip] = Op::Jump(end);
            }
            Statement::ForEach { name, over, body } => {
                let payload = self.event_name(over);
                self.expression(code, frame, over)?;
                frame.open();
                let list = frame.declare("");
                let cursor = frame.declare("");
                let bound = frame.declare(name);
                if let Some(event) = payload {
                    frame.payloads.push((bound, event));
                }
                code.push(Op::PopLocal(list));
                code.push(Op::PushNumber(0.0));
                code.push(Op::PopLocal(cursor));
                let start = code.len() as u32;
                let site = self.plan.loops.len() as u16;
                self.plan.loops.push(LoopSite {
                    list,
                    cursor,
                    name: bound,
                    done: 0,
                });
                code.push(Op::ForNext(site));
                frame.loops.push(Vec::new());
                self.block(code, frame, body)?;
                code.push(Op::Jump(start));
                let done = code.len() as u32;
                self.plan.loops[site as usize].done = done;
                if let Some(marks) = frame.loops.pop() {
                    for mark in marks {
                        code[mark] = Op::Jump(done);
                    }
                }
                frame.close();
            }
            Statement::Break => {
                let mark = code.len();
                code.push(Op::Jump(0));
                if let Some(marks) = frame.loops.last_mut() {
                    marks.push(mark);
                }
            }
            Statement::Stop => code.push(Op::Halt),
        }
        Ok(())
    }

    fn sort(&self, name: &str, columns: &mut Vec<String>, blocks: &mut Vec<String>) {
        let stem = name.split('.').next().unwrap_or(name).to_string();
        let held = if self.column_keys.contains_key(&stem) {
            columns
        } else {
            blocks
        };
        if !held.contains(&stem) {
            held.push(stem);
        }
    }

    fn wanted(&self, body: &[Statement], asks: &mut Vec<String>) {
        for statement in body {
            match statement {
                Statement::When {
                    condition,
                    then,
                    otherwise,
                } => {
                    self.calls(condition, asks);
                    self.wanted(then, asks);
                    self.wanted(otherwise, asks);
                }
                Statement::ForEach { over, body, .. } => {
                    self.calls(over, asks);
                    self.wanted(body, asks);
                }
                Statement::Let { value, .. }
                | Statement::Emit { value, .. }
                | Statement::Send { value, .. }
                | Statement::Assign { value, .. }
                | Statement::Add { value, .. }
                | Statement::Increase { value, .. } => self.calls(value, asks),
                Statement::Remove { subject } => self.calls(subject, asks),
                Statement::Spawn { bindings } => {
                    for (_, value) in bindings {
                        self.calls(value, asks);
                    }
                }
                Statement::Keep { .. } | Statement::Recall { .. } => {}
                Statement::Break | Statement::Stop => {}
            }
        }
    }

    fn calls(&self, expression: &Expression, asks: &mut Vec<String>) {
        match expression {
            Expression::Call { name, arguments } => {
                if self.query_index.contains_key(name) && !asks.contains(name) {
                    asks.push(name.clone());
                }
                for argument in arguments {
                    self.calls(argument, asks);
                }
            }
            Expression::Field { of, .. } | Expression::Unary { value: of, .. } => {
                self.calls(of, asks)
            }
            Expression::IsSome(of) => self.calls(of, asks),
            Expression::Index { of, at } => {
                self.calls(of, asks);
                self.calls(at, asks);
            }
            Expression::Binary { left, right, .. } => {
                self.calls(left, asks);
                self.calls(right, asks);
            }
            Expression::Tuple(parts) | Expression::List(parts) => {
                for part in parts {
                    self.calls(part, asks);
                }
            }
            Expression::Record(fields) => {
                for (_, part) in fields {
                    self.calls(part, asks);
                }
            }
            _ => {}
        }
    }

    fn event_name(&self, over: &Expression) -> Option<String> {
        match over {
            Expression::Name(name) if self.event_keys.contains_key(name) => Some(name.clone()),
            _ => None,
        }
    }

    fn read_place(
        &mut self,
        code: &mut Vec<Op>,
        frame: &mut Frame,
        place: &Place,
    ) -> Result<(), String> {
        if place.subject.is_none()
            && place.field.is_none()
            && let Some(slot) = frame.find(&place.name)
        {
            code.push(Op::PushLocal(slot));
            return Ok(());
        }
        if let Some(field) = &place.field
            && self.is_block(&place.name)
        {
            let slot = self.field_slot(&place.name, field);
            code.push(Op::PushField(slot));
            return Ok(());
        }
        let key = self.column_key(&place.name);
        match &place.subject {
            Some(subject) => {
                self.expression(code, frame, subject)?;
                code.push(Op::PushColumnOf(key));
            }
            None if self.plain(&place.name) => code.push(Op::PushNumberColumn(key)),
            None => code.push(Op::PushColumn(key)),
        }
        Ok(())
    }

    fn flat_columns(&self) -> Vec<bool> {
        self.plan
            .columns
            .iter()
            .map(|name| {
                matches!(
                    self.forms.get(name),
                    Some(Form::Number) | Some(Form::Truth) | Some(Form::Vector) | Some(Form::Thing)
                )
            })
            .collect()
    }

    fn plain(&self, name: &str) -> bool {
        self.forms.get(name).copied() == Some(Form::Number)
    }

    fn write_place(
        &mut self,
        code: &mut Vec<Op>,
        frame: &mut Frame,
        place: &Place,
    ) -> Result<(), String> {
        if place.subject.is_none()
            && place.field.is_none()
            && let Some(slot) = frame.find(&place.name)
        {
            code.push(Op::PopLocal(slot));
            return Ok(());
        }
        if let Some(field) = &place.field
            && self.is_block(&place.name)
        {
            let slot = self.field_slot(&place.name, field);
            note(&mut frame.sets, &format!("{}.{field}", place.name));
            code.push(Op::StoreField(slot));
            return Ok(());
        }
        note(&mut frame.sets, &place.name);
        let key = self.column_key(&place.name);
        let axis = place.field.as_deref().map(axis_index);
        match (&place.subject, axis) {
            (Some(subject), Some(index)) => {
                self.expression(code, frame, subject)?;
                code.push(Op::StoreAxisOf(key, index as u8));
            }
            (Some(subject), None) => {
                self.expression(code, frame, subject)?;
                code.push(Op::StoreColumnOf(key));
            }
            (None, Some(index)) => code.push(Op::StoreAxis(key, index as u8)),
            (None, None) if self.plain(&place.name) => code.push(Op::StoreNumberColumn(key)),
            (None, None) => code.push(Op::StoreColumn(key)),
        }
        Ok(())
    }

    fn poke_place(
        &mut self,
        code: &mut Vec<Op>,
        frame: &mut Frame,
        place: &Place,
    ) -> Result<(), String> {
        if let Some(field) = &place.field
            && self.is_block(&place.name)
        {
            let slot = self.field_slot(&place.name, field);
            note(&mut frame.sets, &format!("{}.{field}", place.name));
            code.push(Op::PokeField(slot));
            return Ok(());
        }
        note(&mut frame.sets, &place.name);
        let key = self.column_key(&place.name);
        match &place.subject {
            Some(subject) => {
                self.expression(code, frame, subject)?;
                code.push(Op::PokeColumnOf(key));
            }
            None => code.push(Op::PokeColumn(key)),
        }
        Ok(())
    }

    fn add_place(
        &mut self,
        code: &mut Vec<Op>,
        frame: &mut Frame,
        place: &Place,
    ) -> Result<(), String> {
        if let Some(field) = &place.field
            && self.is_block(&place.name)
        {
            let slot = self.field_slot(&place.name, field);
            note(&mut frame.adds, &format!("{}.{field}", place.name));
            code.push(Op::AddField(slot));
            return Ok(());
        }
        note(&mut frame.adds, &place.name);
        let key = self.column_key(&place.name);
        let axis = place.field.as_deref().map(axis_index);
        match (&place.subject, axis) {
            (Some(subject), Some(index)) => {
                self.expression(code, frame, subject)?;
                code.push(Op::AddAxisOf(key, index as u8));
            }
            (Some(subject), None) => {
                self.expression(code, frame, subject)?;
                code.push(Op::AddColumnOf(key));
            }
            (None, Some(index)) => code.push(Op::AddAxis(key, index as u8)),
            (None, None) if self.plain(&place.name) => code.push(Op::AddNumberColumn(key)),
            (None, None) => code.push(Op::AddColumn(key)),
        }
        Ok(())
    }

    fn expression(
        &mut self,
        code: &mut Vec<Op>,
        frame: &mut Frame,
        expression: &Expression,
    ) -> Result<(), String> {
        match expression {
            Expression::Number(value) => code.push(Op::PushNumber(*value)),
            Expression::Truth(value) => code.push(Op::PushTruth(*value)),
            Expression::Text(value) => {
                let slot = self.text(value);
                code.push(Op::PushText(slot));
            }
            Expression::Name(name) => {
                if let Some(slot) = frame.find(name) {
                    code.push(Op::PushLocal(slot));
                } else if let Some(key) = self.event_keys.get(name) {
                    code.push(Op::PushEvents(*key));
                } else if self.variants.iter().any(|held| held == name) {
                    let slot = self.text(name);
                    code.push(Op::PushText(slot));
                } else if self.column_keys.contains_key(name) {
                    let key = self.column_key(name);
                    code.push(Op::PushColumn(key));
                } else {
                    code.push(Op::PushNothing);
                }
            }
            Expression::Field { of, name } => {
                if let Some((block, field)) = self.reached(expression) {
                    let slot = self.field_slot(&block, &field);
                    code.push(Op::PushField(slot));
                    return Ok(());
                }
                if let Expression::Name(block) = of.as_ref()
                    && let Some(slot) = frame.find(block)
                    && let Some(event) = frame.payload(slot)
                {
                    let order = self.event_fields.get(event).cloned().unwrap_or_default();
                    let at = order.iter().position(|field| field == name).unwrap_or(0);
                    code.push(Op::PushLocal(slot));
                    code.push(Op::PushNumber(at as f32));
                    code.push(Op::Index);
                    return Ok(());
                }
                self.expression(code, frame, of)?;
                code.push(Op::Axis(axis_index(name) as u8));
            }
            Expression::Index { of, at } => {
                self.expression(code, frame, of)?;
                self.expression(code, frame, at)?;
                code.push(Op::Index);
            }
            Expression::Call { name, arguments } => {
                if self.column_keys.contains_key(name) && arguments.len() == 1 {
                    let key = self.column_key(name);
                    self.expression(code, frame, &arguments[0])?;
                    code.push(Op::PushColumnOf(key));
                    return Ok(());
                }
                for argument in arguments {
                    self.expression(code, frame, argument)?;
                }
                if let Some(index) = self.query_index.get(name) {
                    code.push(Op::CallQuery(*index as u16, arguments.len() as u8));
                    return Ok(());
                }
                match facility::code_of(name) {
                    Some(code_point) => {
                        code.push(Op::Builtin(code_point as u8, arguments.len() as u8))
                    }
                    None => return Err(format!("nothing holds the name {name}")),
                }
            }
            Expression::Unary { operator, value } => {
                self.expression(code, frame, value)?;
                code.push(Op::Unary(*operator));
            }
            Expression::Binary {
                operator,
                left,
                right,
            } => {
                self.expression(code, frame, left)?;
                self.expression(code, frame, right)?;
                code.push(Op::Binary(*operator));
            }
            Expression::Tuple(parts) => {
                for part in parts {
                    self.expression(code, frame, part)?;
                }
                code.push(Op::MakeVector(parts.len() as u8));
            }
            Expression::List(items) => {
                for item in items {
                    self.expression(code, frame, item)?;
                }
                code.push(Op::MakeList(items.len() as u8));
            }
            Expression::Record(fields) => {
                for (_, value) in fields {
                    self.expression(code, frame, value)?;
                }
                code.push(Op::MakeList(fields.len() as u8));
            }
            Expression::IsSome(inner) => {
                self.expression(code, frame, inner)?;
                code.push(Op::IsSome);
            }
        }
        Ok(())
    }
}

fn note(held: &mut Vec<String>, name: &str) {
    if !held.iter().any(|one| one == name) {
        held.push(name.to_string());
    }
}

fn stage(plan: &mut Plan) -> Result<(), String> {
    let mut named: Vec<String> = Vec::new();
    let mut marks: Vec<u16> = Vec::with_capacity(plan.turn.len());
    let mut loose = 0u16;
    for index in &plan.turn {
        let Some(unit) = plan.units.get(*index) else {
            marks.push(u16::MAX);
            continue;
        };
        if unit.phase.is_empty() {
            marks.push(u16::MAX - loose);
            loose += 1;
            continue;
        }
        let at = match named.iter().position(|held| held == &unit.phase) {
            Some(at) => at,
            None => {
                named.push(unit.phase.clone());
                named.len() - 1
            }
        };
        marks.push(at as u16);
    }

    let mut seats: Vec<(usize, u16)> = plan.turn.iter().copied().zip(marks).collect();
    let mut first: Vec<(u16, usize)> = Vec::new();
    for (place, (_, mark)) in seats.iter().enumerate() {
        if !first.iter().any(|(held, _)| held == mark) {
            first.push((*mark, place));
        }
    }
    seats.sort_by_key(|(_, mark)| {
        first
            .iter()
            .find(|(held, _)| held == mark)
            .map(|(_, place)| *place)
            .unwrap_or_default()
    });

    plan.turn = seats.iter().map(|(index, _)| *index).collect();
    plan.phases = seats.iter().map(|(_, mark)| *mark).collect();

    for (mark, _) in &first {
        let members: Vec<usize> = seats
            .iter()
            .filter(|(_, held)| held == mark)
            .map(|(index, _)| *index)
            .collect();
        if members.len() < 2 {
            continue;
        }
        clash(plan, &members)?;
    }
    Ok(())
}

fn clash(plan: &Plan, members: &[usize]) -> Result<(), String> {
    for (order, index) in members.iter().enumerate() {
        let Some(unit) = plan.units.get(*index) else {
            continue;
        };
        for other in &members[order + 1..] {
            let Some(second) = plan.units.get(*other) else {
                continue;
            };
            for name in unit.sets.iter().chain(unit.adds.iter()) {
                let written = second.sets.contains(name) || second.adds.contains(name);
                if !written {
                    continue;
                }
                if !unit.sets.contains(name) && !second.sets.contains(name) {
                    continue;
                }
                return Err(format!(
                    "{} and {} both write {name} in phase {}, and only += shares a value",
                    unit.name, second.name, unit.phase
                ));
            }
        }
    }
    Ok(())
}

fn served(plan: &Plan) -> Result<(), String> {
    for unit in &plan.units {
        for wanted in &unit.needs {
            let met = plan
                .units
                .iter()
                .any(|other| other.serves.iter().any(|held| held == wanted));
            if !met {
                return Err(format!(
                    "{} needs {wanted}, and nothing in this description provides it",
                    unit.name
                ));
            }
        }
    }
    Ok(())
}

fn habits(system: &System, pure: bool) -> Result<(), String> {
    let loose = system.ports.iter().any(|port| match port {
        Port::Select(selection) => {
            selection.loose && selection.cardinality == data::Cardinality::Every
        }
        _ => false,
    });
    let mut marks = Marks::default();
    reads_of(&system.body, &mut marks);
    if loose && marks.halts {
        return Err(format!(
            "{} stops inside a pick that states any order, and which things ran before the stop reads that order",
            system.name
        ));
    }
    if loose && marks.removes {
        return Err(format!(
            "{} removes a thing inside a pick that states any order, and a later thing reads that removal",
            system.name
        ));
    }
    if pure
        && system
            .ports
            .iter()
            .any(|port| matches!(port, Port::Creates | Port::Removes))
    {
        return Err(format!(
            "{} is pure and makes or takes away things, and skipping it would not do that again",
            system.name
        ));
    }
    if pure && marks.draws {
        return Err(format!(
            "{} is pure and draws, and a draw is not a function of what the system reads",
            system.name
        ));
    }
    if !pure {
        return Ok(());
    }
    for port in &system.ports {
        let named = match port {
            Port::Reads(list) | Port::Writes(list) => list.clone(),
            _ => Vec::new(),
        };
        for name in named {
            let block = name.split('.').next().unwrap_or(&name).to_string();
            if data::SENSED.contains(&block.as_str()) {
                return Err(format!(
                    "{} is pure and names {block}, which the frame gives it rather than the description",
                    system.name
                ));
            }
        }
    }
    Ok(())
}

#[derive(Default)]
struct Marks {
    halts: bool,
    removes: bool,
    draws: bool,
}

fn reads_of(body: &[Statement], marks: &mut Marks) {
    for statement in body {
        match statement {
            Statement::Stop => marks.halts = true,
            Statement::Remove { subject } => {
                marks.removes = true;
                drawn(subject, marks);
            }
            Statement::When {
                condition,
                then,
                otherwise,
            } => {
                drawn(condition, marks);
                reads_of(then, marks);
                reads_of(otherwise, marks);
            }
            Statement::ForEach { over, body, .. } => {
                drawn(over, marks);
                reads_of(body, marks);
            }
            Statement::Let { value, .. }
            | Statement::Assign { value, .. }
            | Statement::Add { value, .. }
            | Statement::Increase { value, .. }
            | Statement::Emit { value, .. }
            | Statement::Send { value, .. } => drawn(value, marks),
            Statement::Spawn { bindings } => {
                for (_, value) in bindings {
                    drawn(value, marks);
                }
            }
            Statement::Keep { .. } | Statement::Recall { .. } => {}
            Statement::Break => {}
        }
    }
}

fn drawn(expression: &Expression, marks: &mut Marks) {
    match expression {
        Expression::Call { name, arguments } => {
            if name == "draw" {
                marks.draws = true;
            }
            for argument in arguments {
                drawn(argument, marks);
            }
        }
        Expression::Field { of, .. } | Expression::Unary { value: of, .. } => drawn(of, marks),
        Expression::IsSome(of) => drawn(of, marks),
        Expression::Index { of, at } => {
            drawn(of, marks);
            drawn(at, marks);
        }
        Expression::Binary { left, right, .. } => {
            drawn(left, marks);
            drawn(right, marks);
        }
        Expression::Tuple(parts) | Expression::List(parts) => {
            for part in parts {
                drawn(part, marks);
            }
        }
        Expression::Record(held) => {
            for (_, value) in held {
                drawn(value, marks);
            }
        }
        Expression::Number(_)
        | Expression::Text(_)
        | Expression::Truth(_)
        | Expression::Name(_) => {}
    }
}

impl Maker {
    fn carried(&self, name: &str, value: &Expression) -> Result<(), String> {
        let Some(wanted) = self.declared.get(name) else {
            return Ok(());
        };
        let held: Vec<String> = match value {
            Expression::Record(fields) => fields.iter().map(|(field, _)| field.clone()).collect(),
            _ => Vec::new(),
        };
        for field in &held {
            if !wanted.iter().any(|one| one == field) {
                return Err(format!(
                    "{name} carries no {field}, and its declaration says what it carries"
                ));
            }
        }
        for field in wanted {
            if !held.iter().any(|one| one == field) {
                return Err(format!(
                    "{name} carries {field}, and this one leaves it out"
                ));
            }
        }
        Ok(())
    }
}

#[derive(Default)]
struct Traffic {
    sends: Vec<String>,
    emits: Vec<String>,
    hears: Vec<String>,
}

fn routed(program: &Program) -> Result<(), String> {
    let mut seen: Vec<(String, Traffic)> = Vec::new();
    for system in &program.systems {
        let mut traffic = Traffic::default();
        traffic_of(&system.body, &mut traffic);
        for name in &traffic.sends {
            if !program.commands.iter().any(|held| held == name) {
                return Err(format!(
                    "{} sends {name}, and {name} is an event, so emit it",
                    system.name
                ));
            }
        }
        for name in &traffic.emits {
            if program.commands.iter().any(|held| held == name) {
                return Err(format!(
                    "{} emits {name}, and {name} is a command, so send it",
                    system.name
                ));
            }
        }
        seen.push((system.name.clone(), traffic));
    }

    for name in &program.commands {
        let senders: Vec<&String> = seen
            .iter()
            .filter(|(_, traffic)| traffic.sends.iter().any(|held| held == name))
            .map(|(who, _)| who)
            .collect();
        let handlers: Vec<&String> = seen
            .iter()
            .filter(|(_, traffic)| traffic.hears.iter().any(|held| held == name))
            .map(|(who, _)| who)
            .collect();
        if senders.is_empty() {
            return Err(format!(
                "nothing sends {name}, and a command with no sender is dead"
            ));
        }
        match handlers.len() {
            0 => {
                return Err(format!(
                    "nothing handles {name}, and a command has to reach one system"
                ));
            }
            1 => {}
            _ => {
                return Err(format!(
                    "{} and {} both handle {name}, and a command reaches one system",
                    handlers[0], handlers[1]
                ));
            }
        }
    }
    Ok(())
}

fn traffic_of(body: &[Statement], traffic: &mut Traffic) {
    for statement in body {
        match statement {
            Statement::Send { name, value } => {
                traffic.sends.push(name.clone());
                heard(value, traffic);
            }
            Statement::Emit { name, value } => {
                traffic.emits.push(name.clone());
                heard(value, traffic);
            }
            Statement::When {
                condition,
                then,
                otherwise,
            } => {
                heard(condition, traffic);
                traffic_of(then, traffic);
                traffic_of(otherwise, traffic);
            }
            Statement::ForEach { over, body, .. } => {
                heard(over, traffic);
                traffic_of(body, traffic);
            }
            Statement::Let { value, .. }
            | Statement::Assign { value, .. }
            | Statement::Add { value, .. }
            | Statement::Increase { value, .. } => heard(value, traffic),
            Statement::Remove { subject } => heard(subject, traffic),
            Statement::Spawn { bindings } => {
                for (_, value) in bindings {
                    heard(value, traffic);
                }
            }
            Statement::Keep { .. } | Statement::Recall { .. } => {}
            Statement::Break | Statement::Stop => {}
        }
    }
}

fn heard(expression: &Expression, traffic: &mut Traffic) {
    match expression {
        Expression::Name(name) => traffic.hears.push(name.clone()),
        Expression::Call { arguments, .. } => {
            for argument in arguments {
                heard(argument, traffic);
            }
        }
        Expression::Field { of, .. } | Expression::Unary { value: of, .. } => heard(of, traffic),
        Expression::IsSome(of) => heard(of, traffic),
        Expression::Index { of, at } => {
            heard(of, traffic);
            heard(at, traffic);
        }
        Expression::Binary { left, right, .. } => {
            heard(left, traffic);
            heard(right, traffic);
        }
        Expression::Tuple(parts) | Expression::List(parts) => {
            for part in parts {
                heard(part, traffic);
            }
        }
        Expression::Record(held) => {
            for (_, value) in held {
                heard(value, traffic);
            }
        }
        Expression::Number(_) | Expression::Text(_) | Expression::Truth(_) => {}
    }
}

fn literal_form(value: &Expression) -> Option<Form> {
    match value {
        Expression::Number(_) => Some(Form::Number),
        Expression::Truth(_) => Some(Form::Truth),
        Expression::Text(_) => Some(Form::Text),
        Expression::Tuple(_) => Some(Form::Vector),
        Expression::List(_) => Some(Form::List),
        _ => None,
    }
}

fn widest(left: Option<Form>, right: Option<Form>) -> Option<Form> {
    if left == Some(Form::Vector) || right == Some(Form::Vector) {
        return Some(Form::Vector);
    }
    match (left, right) {
        (Some(Form::Number), Some(Form::Number)) => Some(Form::Number),
        _ => None,
    }
}

impl Maker {
    fn fits(&self, frame: &Frame, place: &Place, value: &Expression) -> Result<(), String> {
        if place.index.is_some() {
            return Ok(());
        }
        if place.subject.is_none() && place.field.is_none() && frame.find(&place.name).is_some() {
            return Ok(());
        }
        let wanted = match &place.field {
            Some(field) if self.is_block(&place.name) => self
                .field_forms
                .get(&(place.name.clone(), field.clone()))
                .copied(),
            Some(_) => Some(Form::Number),
            None => self.forms.get(&place.name).copied(),
        };
        let named = match &place.field {
            Some(field) => format!("{}.{field}", place.name),
            None => place.name.clone(),
        };
        self.matches(frame, &named, wanted, value)
    }

    fn matches(
        &self,
        frame: &Frame,
        named: &str,
        wanted: Option<Form>,
        value: &Expression,
    ) -> Result<(), String> {
        let (Some(wanted), Some(held)) = (wanted, self.shape_of(frame, value)) else {
            return Ok(());
        };
        if form::agrees(wanted, held) {
            return Ok(());
        }
        Err(format!(
            "{named} holds {}, and this gives it {}",
            form::name_of(wanted),
            form::name_of(held)
        ))
    }

    fn shape_of(&self, frame: &Frame, expression: &Expression) -> Option<Form> {
        match expression {
            Expression::Number(_) => Some(Form::Number),
            Expression::Truth(_) => Some(Form::Truth),
            Expression::Text(_) => Some(Form::Text),
            Expression::Tuple(_) => Some(Form::Vector),
            Expression::List(_) | Expression::Record(_) => Some(Form::List),
            Expression::IsSome(_) => Some(Form::Truth),
            Expression::Unary { operator, value } => match operator {
                data::UnaryOperator::Not => Some(Form::Truth),
                data::UnaryOperator::Negate => self.shape_of(frame, value),
            },
            Expression::Binary {
                operator,
                left,
                right,
            } => match operator {
                data::BinaryOperator::Range => Some(Form::List),
                data::BinaryOperator::And
                | data::BinaryOperator::Or
                | data::BinaryOperator::Equal
                | data::BinaryOperator::NotEqual
                | data::BinaryOperator::Less
                | data::BinaryOperator::LessOrEqual
                | data::BinaryOperator::Greater
                | data::BinaryOperator::GreaterOrEqual => Some(Form::Truth),
                data::BinaryOperator::Add => {
                    let one = self.shape_of(frame, left);
                    let two = self.shape_of(frame, right);
                    if one == Some(Form::Text) || two == Some(Form::Text) {
                        return Some(Form::Text);
                    }
                    widest(one, two)
                }
                data::BinaryOperator::Subtract
                | data::BinaryOperator::Multiply
                | data::BinaryOperator::Divide => {
                    widest(self.shape_of(frame, left), self.shape_of(frame, right))
                }
            },
            Expression::Name(name) => {
                if let Some(slot) = frame.find(name) {
                    return frame.form(slot);
                }
                if self.event_keys.contains_key(name) {
                    return Some(Form::List);
                }
                if self.variants.iter().any(|held| held == name) {
                    return Some(Form::Text);
                }
                self.forms.get(name).copied()
            }
            Expression::Field { of, name } => {
                let Expression::Name(stem) = of.as_ref() else {
                    return None;
                };
                if self.is_block(stem) {
                    return self.field_forms.get(&(stem.clone(), name.clone())).copied();
                }
                if self.forms.get(stem) == Some(&Form::Vector)
                    && matches!(name.as_str(), "x" | "y" | "z" | "w")
                {
                    return Some(Form::Number);
                }
                None
            }
            Expression::Index { .. } => None,
            Expression::Call { name, .. } => {
                if self.query_index.contains_key(name) {
                    return None;
                }
                match facility::code_of(name) {
                    Some(11) => None,
                    Some(20) => Some(Form::List),
                    Some(_) => Some(Form::Number),
                    None => None,
                }
            }
        }
    }
}

fn broad(code: &[Op], picks: &[Pick], flat: &[bool]) -> bool {
    use crate::data::{BinaryOperator, Cardinality};
    let Some(pick) = picks.first() else {
        return false;
    };
    if picks.len() != 1 || pick.cardinality != Cardinality::Every {
        return false;
    }
    let label = pick.label;
    let holds = |key: u16| flat.get(key as usize).copied().unwrap_or(false);
    let mut gates: Vec<(usize, bool)> = Vec::new();
    let mut at = 0usize;
    while at < code.len() {
        while gates.last().is_some_and(|(target, _)| *target == at) {
            gates.pop();
        }
        let op = code[at];
        at += 1;
        let fine = match op {
            Op::PushNumber(_) | Op::PushTruth(_) | Op::PushNothing | Op::Pop => true,
            Op::PushField(_) | Op::StoreField(_) | Op::AddField(_) | Op::StoreAxisField(_, _) => {
                true
            }
            Op::PushLocal(slot) => slot != label,
            Op::PopLocal(slot) => slot != label,
            Op::PushColumn(key) | Op::PushNumberColumn(key) => holds(key),
            Op::StoreColumn(key) | Op::StoreNumberColumn(key) => holds(key),
            Op::AddColumn(key) | Op::AddNumberColumn(key) => holds(key),
            Op::StoreAxis(key, _) | Op::AddAxis(key, _) => holds(key),
            Op::Axis(_) | Op::Unary(_) | Op::IsSome | Op::MakeVector(_) => true,
            Op::Binary(operator) => operator != BinaryOperator::Range,
            Op::MixNumber(operator, _) => operator != BinaryOperator::Range,
            Op::MixField(operator, _) => operator != BinaryOperator::Range,
            Op::MixColumn(operator, key) => operator != BinaryOperator::Range && holds(key),
            Op::Builtin(which, _) => which != 12 && which != 13 && which != 20,
            Op::JumpIfFalse(target) => {
                let target = target as usize;
                if target < at || target > code.len() {
                    return false;
                }
                gates.push((target, true));
                true
            }
            Op::Jump(target) => {
                let target = target as usize;
                let Some(last) = gates.last_mut() else {
                    return false;
                };
                if !last.1 || last.0 != at || target < at || target > code.len() {
                    return false;
                }
                last.1 = false;
                last.0 = target;
                true
            }
            _ => false,
        };
        if !fine {
            return false;
        }
    }
    gates.iter().all(|(target, _)| *target >= code.len())
}

fn fuse(code: &[Op]) -> (Vec<Op>, Vec<u32>) {
    let mut place = vec![0u32; code.len() + 1];
    let mut made: Vec<Op> = Vec::with_capacity(code.len());
    let mut at = 0usize;
    while at < code.len() {
        place[at] = made.len() as u32;
        let Some(Op::Binary(operator)) = code.get(at + 1).copied() else {
            made.push(code[at]);
            at += 1;
            continue;
        };
        let fused = match code[at] {
            Op::PushNumber(value) => Some(Op::MixNumber(operator, value)),
            Op::PushField(slot) => Some(Op::MixField(operator, slot)),
            Op::PushColumn(key) | Op::PushNumberColumn(key) => Some(Op::MixColumn(operator, key)),
            _ => None,
        };
        match fused {
            Some(held) => {
                made.push(held);
                place[at + 1] = made.len() as u32;
                at += 2;
            }
            None => {
                made.push(code[at]);
                at += 1;
            }
        }
    }
    place[code.len()] = made.len() as u32;
    for op in made.iter_mut() {
        match op {
            Op::Jump(target) => *target = place[*target as usize],
            Op::JumpIfFalse(target) => *target = place[*target as usize],
            _ => {}
        }
    }
    (made, place)
}
