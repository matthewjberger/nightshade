use crate::commands::edit;
use crate::data::edit::{FORMS, KINDS, OPERATORS, PORTS, Spot, TABS, Touch};
use crate::data::facility;
use crate::data::{self, Expression, Port, Program, Statement};
use crate::queries::{lesson, shape, write};
use crate::resources::{Script, Store, Studio, Switcher};
use nightshade::prelude::*;
use nightshade_ui::components::layout_types::UiLayoutType;
use nightshade_ui::prelude::*;

const DIM: [f32; 4] = [0.42, 0.47, 0.58, 1.0];
const CODE: [f32; 4] = [0.80, 0.84, 0.94, 1.0];
const WRITE: [f32; 4] = [1.0, 0.62, 0.20, 1.0];
const SHARED: [f32; 4] = [0.32, 0.86, 0.56, 1.0];
const FAULT: [f32; 4] = [1.0, 0.42, 0.42, 1.0];
const AWAY: f32 = 30.0;
const KIND_WIDTH: f32 = 96.0;

struct Pen {
    controls: Vec<(Touch, Entity)>,
    grid: Entity,
    input: Option<Entity>,
}

pub fn build(
    script: Res<Script>,
    switcher: Res<Switcher>,
    store: Res<Store>,
    mut studio_state: ResMut<Studio>,
    mut build: nightshade_ui::commands::builder::UiBuild,
) {
    let wanted = studio_state.stale || studio_state.shown != switcher.playing;
    if !wanted {
        return;
    }

    let shown = switcher.playing.clone();
    let previous = studio_state.root;
    let tab = studio_state.tab;
    let chosen = studio_state.chosen;
    let form = studio_state.form;
    let port = studio_state.port;
    let step = studio_state.step;
    let name = studio_state.name.clone();
    let program = script.source.clone();
    let plan = script.plan.clone();
    let fault = script.fault.clone();
    let cards = shape::cards(&plan, &store);

    let made = {
        let context = build.context();
        if let Some(root) = previous {
            context.commands.despawn_recursive(root);
        }
        let mut tree = UiTreeBuilder::new(context);

        let docked = tree.add_docked_panel_right("studio", "Studio", data::STUDIO_WIDTH);
        let content = tree
            .component::<UiPanelData>(docked)
            .map(|data| data.content_entity)
            .expect("the panel has a content node");
        tree.edit::<UiLayoutNode>(content, |node| {
            node.flow_layout = Some(FlowLayout {
                direction: FlowDirection::Vertical,
                padding: data::PANEL_PADDING,
                spacing: 6.0,
                alignment: FlowAlignment::Start,
                cross_alignment: FlowAlignment::Start,
                wrap: false,
            });
        });

        let mut graph = None;
        let mut pen = Pen {
            controls: Vec::new(),
            grid: content,
            input: None,
        };

        let bar = tree.in_parent(content, |tree| tree.add_tab_bar(TABS, tab));
        tree.edit::<UiLayoutNode>(bar, |node| {
            node.flow_child_size =
                Some(Rl(Vec2::new(100.0, 0.0)) + Ab(Vec2::new(0.0, data::TAB_HEIGHT)));
            node.flex_grow = Some(0.0);
        });
        pen.controls.push((Touch::Tab, bar));

        let area = tree.in_parent(content, |tree| tree.add_scroll_area_fill(2.0, 4.0));
        tree.edit::<UiLayoutNode>(area, |node| {
            if let Some(UiLayoutType::Boundary(boundary)) = node.base_layout.as_mut() {
                boundary.position_1 = Ab(Vec2::new(0.0, data::TAB_HEIGHT + 8.0)).into();
                boundary.position_2 = Rl(Vec2::new(100.0, 100.0)).into();
            }
        });
        let inner = tree
            .component::<UiScrollAreaData>(area)
            .map(|data| data.content_entity)
            .expect("the scroll area has a content node");

        pen.grid = tree.in_parent(inner, |tree| tree.add_property_grid(data::LABEL_WIDTH));

        if let Some(fault) = &fault {
            tree.in_parent(inner, |tree| {
                for line in fold(fault, data::LINE_WIDTH) {
                    chip(tree, &line, FAULT);
                }
            });
        }
        match tab {
            0 => shop(
                &mut tree, &mut pen, inner, &program, chosen, form, port, &name,
            ),
            1 => {
                graph = Some(tree.in_parent(inner, |tree| {
                    crate::systems::inspect::draw(tree, &plan, &cards, chosen, &mut pen.controls)
                }));
            }
            2 => school(&mut tree, &mut pen, inner, &program, step),
            _ => {
                tree.in_parent(inner, |tree| paper(tree, &program));
            }
        }

        tree.runtime_mut().visible = true;
        let root = tree.finish();
        (root, pen.controls, pen.input, graph)
    };

    let studio = &mut *studio_state;
    studio.root = Some(made.0);
    studio.controls = made.1;
    studio.input = made.2;
    studio.graph = made.3;
    studio.shown = shown;
    studio.stale = false;
}

fn shop(
    tree: &mut UiTreeBuilder,
    pen: &mut Pen,
    home: Entity,
    program: &Program,
    chosen: Option<usize>,
    form: usize,
    port: usize,
    name: &str,
) {
    let area = tree.add_property_row(pen.grid, home, "name");
    let input = tree.in_parent(area, |tree| {
        tree.add_text_input_with_value("a name for the next thing", name)
    });
    pen.input = Some(input);

    let things = section(tree, home, "Things", true);
    tree.in_parent(things, |tree| {
        for (block, source, _, rows) in &program.assets {
            chip(tree, &format!("{block} from {source}"), DIM);
            for row in rows {
                chip(tree, &format!("   {} is {}", row.label, row.file), CODE);
            }
        }
    });
    for (at, held) in program.components.iter().enumerate() {
        let row = tree.in_parent(things, |tree| tree.add_row());
        let made = tree.in_parent(row, |tree| {
            wide(tree, held, CODE);
            tree.add_icon_button(icons::material::CLOSE, data::ICON_SIZE)
        });
        hold(tree, made, AWAY);
        pen.controls.push((Touch::DropComponent(at), made));
    }
    let made = tree.in_parent(things, |tree| tree.add_button("add a component"));
    pen.controls.push((Touch::AddComponent, made));

    for block in 0..crate::queries::edit::count(program) {
        let name = crate::queries::edit::block_name(program, block);
        let heading = if block < program.configs.len() {
            format!("Numbers of {name}")
        } else {
            format!("Shared {name}")
        };
        let here = section(tree, home, &heading, block == 0);
        let fields = crate::queries::edit::fields(program, block).to_vec();
        record(tree, pen, here, program, block, &fields, "", 0);
    }

    let here = section(tree, home, "Systems", true);
    let made = tree.in_parent(here, |tree| tree.add_button("add a system"));
    pen.controls.push((Touch::AddSystem, made));

    for (at, system) in program.systems.iter().enumerate() {
        let row = tree.in_parent(here, |tree| tree.add_row());
        let made = tree.in_parent(row, |tree| {
            let mark = if chosen == Some(at) {
                format!("[{}]", system.name)
            } else {
                system.name.clone()
            };
            (
                tree.add_button(&mark),
                tree.add_icon_button(icons::material::CLOSE, data::ICON_SIZE),
            )
        });
        hold(tree, made.1, AWAY);
        pen.controls.push((Touch::PickSystem(at), made.0));
        pen.controls.push((Touch::DropSystem(at), made.1));
        if chosen != Some(at) {
            continue;
        }

        let held = section(tree, here, "Ports", true);
        for (which, gate) in system.ports.iter().enumerate() {
            let row = tree.in_parent(held, |tree| tree.add_row());
            let names = knowable(program);
            let now = names
                .iter()
                .position(|one| one == &named(gate))
                .unwrap_or(0);
            let made = tree.in_parent(row, |tree| {
                wide(tree, &way(gate), SHARED);
                let pick = match gate {
                    Port::Reads(_) | Port::Writes(_) | Port::Select(_) => {
                        let list: Vec<&str> = names.iter().map(|one| one.as_str()).collect();
                        Some(tree.add_dropdown_searchable(&list, now))
                    }
                    _ => None,
                };
                let away = tree.add_icon_button(icons::material::CLOSE, data::ICON_SIZE);
                (pick, away)
            });
            if let Some(pick) = made.0 {
                let held = skin(tree, pick);
                hold(tree, held, data::PICK_WIDTH);
                pen.controls.push((Touch::PortName(at, which), pick));
            }
            hold(tree, made.1, AWAY);
            pen.controls.push((Touch::DropPort(at, which), made.1));
        }
        let row = tree.in_parent(held, |tree| tree.add_row());
        let made = tree.in_parent(row, |tree| {
            (
                tree.add_dropdown(PORTS, port),
                tree.add_icon_button(icons::material::ADD, data::ICON_SIZE),
            )
        });
        hold(tree, made.1, AWAY);
        pen.controls.push((Touch::PortKind(at), made.0));
        pen.controls.push((Touch::AddPort(at, 0), made.1));

        let body = section(tree, here, "Behavior", true);
        let area = tree.add_property_row(pen.grid, body, "new one");
        let made = tree.in_parent(area, |tree| tree.add_dropdown(FORMS, form));
        let held = skin(tree, made);
        fill(tree, held);
        pen.controls.push((Touch::NewForm, made));
        story(tree, pen, body, program, at, &system.body, &[], 0);
    }
}

fn record(
    tree: &mut UiTreeBuilder,
    pen: &mut Pen,
    home: Entity,
    program: &Program,
    block: usize,
    fields: &[crate::data::Field],
    path: &str,
    depth: usize,
) {
    for (at, field) in fields.iter().enumerate() {
        let Some(rest) = under(&field.name, path) else {
            continue;
        };
        if rest.contains('.') {
            continue;
        }
        let Some(held) = &field.default else {
            continue;
        };
        let spot = Spot::value(block, at);
        let area = worth(tree, pen, home, program, &rest, held, &spot, 0);
        let made = tree.in_parent(area, |tree| {
            tree.add_icon_button(icons::material::CLOSE, 12.0)
        });
        hold(tree, made, 20.0);
        pen.controls.push((Touch::DropField(block, at), made));
    }

    let row = tree.in_parent(home, |tree| tree.add_row());
    let made = tree.in_parent(row, |tree| {
        (
            tree.add_button("add a value"),
            tree.add_button("add a group"),
        )
    });
    share(tree, made.0, 49.0);
    share(tree, made.1, 49.0);
    pen.controls
        .push((Touch::AddField(block, path.to_string()), made.0));
    pen.controls
        .push((Touch::AddGroup(block, path.to_string()), made.1));

    let mut seen: Vec<String> = Vec::new();
    for field in fields {
        let Some(rest) = under(&field.name, path) else {
            continue;
        };
        let Some((head, _)) = rest.split_once('.') else {
            continue;
        };
        if seen.iter().any(|one| one == head) {
            continue;
        }
        seen.push(head.to_string());
        let next = if path.is_empty() {
            head.to_string()
        } else {
            format!("{path}.{head}")
        };
        let here = section(tree, home, head, depth < 1);
        record(tree, pen, here, program, block, fields, &next, depth + 1);
    }
}

fn under(name: &str, path: &str) -> Option<String> {
    if path.is_empty() {
        return Some(name.to_string());
    }
    name.strip_prefix(&format!("{path}."))
        .map(|rest| rest.to_string())
}

fn story(
    tree: &mut UiTreeBuilder,
    pen: &mut Pen,
    home: Entity,
    program: &Program,
    system: usize,
    body: &[Statement],
    trail: &[u16],
    depth: usize,
) {
    for (at, statement) in body.iter().enumerate() {
        let mut here = trail.to_vec();
        here.push(at as u16);
        let lead = "   ".repeat(depth);
        let form = FORMS[data::edit::form_of(statement)];

        let row = tree.in_parent(home, |tree| tree.add_row());
        let made = tree.in_parent(row, |tree| {
            wide(tree, &format!("{lead}{form}"), WRITE);
            (
                tree.add_icon_button(icons::material::ARROW_DROP_UP, data::ICON_SIZE),
                tree.add_icon_button(icons::material::ARROW_DROP_DOWN, data::ICON_SIZE),
                tree.add_icon_button(icons::material::CLOSE, data::ICON_SIZE),
            )
        });
        hold(tree, made.0, AWAY);
        hold(tree, made.1, AWAY);
        hold(tree, made.2, AWAY);
        pen.controls
            .push((Touch::LiftStatement(system, here.clone(), false), made.0));
        pen.controls
            .push((Touch::LiftStatement(system, here.clone(), true), made.1));
        pen.controls
            .push((Touch::DropStatement(system, here.clone()), made.2));

        match statement {
            Statement::Assign { place, value }
            | Statement::Add { place, value }
            | Statement::Increase { place, value } => {
                place_row(tree, pen, home, program, system, &here, &write::spot(place));
                worth(
                    tree,
                    pen,
                    home,
                    program,
                    "to",
                    value,
                    &Spot::body(system, &here, 0),
                    depth + 1,
                );
            }
            Statement::Let { name, value } => {
                place_row(tree, pen, home, program, system, &here, name);
                worth(
                    tree,
                    pen,
                    home,
                    program,
                    "is",
                    value,
                    &Spot::body(system, &here, 0),
                    depth + 1,
                );
            }
            Statement::Keep { .. } | Statement::Recall { .. } => {}
            Statement::Emit { name, value } | Statement::Send { name, value } => {
                place_row(tree, pen, home, program, system, &here, name);
                worth(
                    tree,
                    pen,
                    home,
                    program,
                    "with",
                    value,
                    &Spot::body(system, &here, 0),
                    depth + 1,
                );
            }
            Statement::Remove { subject } => {
                worth(
                    tree,
                    pen,
                    home,
                    program,
                    "what",
                    subject,
                    &Spot::body(system, &here, 0),
                    depth + 1,
                );
            }
            Statement::When {
                condition,
                then,
                otherwise,
            } => {
                worth(
                    tree,
                    pen,
                    home,
                    program,
                    "if",
                    condition,
                    &Spot::body(system, &here, 0),
                    depth + 1,
                );
                story(tree, pen, home, program, system, then, &here, depth + 1);
                more(tree, pen, home, system, &here, depth + 1);
                if !otherwise.is_empty() {
                    tree.in_parent(home, |tree| {
                        chip(tree, &format!("{lead}   otherwise"), WRITE);
                    });
                    let mut other = here.clone();
                    let last = other.len() - 1;
                    other[last] |= data::edit::ELSE;
                    story(
                        tree,
                        pen,
                        home,
                        program,
                        system,
                        otherwise,
                        &other,
                        depth + 1,
                    );
                }
            }
            Statement::ForEach { name, over, body } => {
                place_row(tree, pen, home, program, system, &here, name);
                worth(
                    tree,
                    pen,
                    home,
                    program,
                    "in",
                    over,
                    &Spot::body(system, &here, 0),
                    depth + 1,
                );
                story(tree, pen, home, program, system, body, &here, depth + 1);
                more(tree, pen, home, system, &here, depth + 1);
            }
            Statement::Spawn { bindings } => {
                for (slot, (name, value)) in bindings.iter().enumerate() {
                    let names = spawnable(program);
                    let list: Vec<&str> = names.iter().map(|one| one.as_str()).collect();
                    let now = names.iter().position(|one| one == name).unwrap_or(0);
                    let area = tree.add_property_row(pen.grid, home, &format!("{lead}   with"));
                    let made =
                        tree.in_parent(area, |tree| tree.add_dropdown_searchable(&list, now));
                    let held = skin(tree, made);
                    hold(tree, held, data::PICK_WIDTH);
                    pen.controls
                        .push((Touch::Binding(system, here.clone(), slot), made));
                    worth(
                        tree,
                        pen,
                        home,
                        program,
                        "",
                        value,
                        &Spot::body(system, &here, slot as u16),
                        depth + 2,
                    );
                }
                let row = tree.in_parent(home, |tree| tree.add_row());
                let made = tree.in_parent(row, |tree| tree.add_button("one more part"));
                pen.controls
                    .push((Touch::AddBinding(system, here.clone()), made));
            }
            Statement::Break | Statement::Stop => {}
        }
    }
    more(tree, pen, home, system, trail, depth);
}

fn more(
    tree: &mut UiTreeBuilder,
    pen: &mut Pen,
    home: Entity,
    system: usize,
    trail: &[u16],
    depth: usize,
) {
    let lead = "   ".repeat(depth);
    let row = tree.in_parent(home, |tree| tree.add_row());
    let made = tree.in_parent(row, |tree| {
        wide(tree, &lead, DIM);
        tree.add_icon_button(icons::material::ADD, data::ICON_SIZE)
    });
    hold(tree, made, AWAY);
    pen.controls
        .push((Touch::AddStatement(system, trail.to_vec(), 0), made));
}

fn place_row(
    tree: &mut UiTreeBuilder,
    pen: &mut Pen,
    home: Entity,
    program: &Program,
    system: usize,
    trail: &[u16],
    now: &str,
) {
    let names = placeable(program, now);
    let list: Vec<&str> = names.iter().map(|one| one.as_str()).collect();
    let at = names.iter().position(|one| one == now).unwrap_or(0);
    let area = tree.add_property_row(pen.grid, home, "what");
    let made = tree.in_parent(area, |tree| tree.add_dropdown_searchable(&list, at));
    let held = skin(tree, made);
    fill(tree, held);
    pen.controls
        .push((Touch::Place(system, trail.to_vec()), made));
}

fn worth(
    tree: &mut UiTreeBuilder,
    pen: &mut Pen,
    home: Entity,
    program: &Program,
    label: &str,
    value: &Expression,
    spot: &Spot,
    depth: usize,
) -> Entity {
    let lead = "   ".repeat(depth.min(4));
    let shown = format!("{lead}{label}");
    let area = tree.add_property_row(pen.grid, home, &shown);

    if let Some(parts) = tint(value) {
        let made = tree.in_parent(area, |tree| {
            tree.add_color_picker(Vec4::new(parts[0], parts[1], parts[2], 1.0))
        });
        fill(tree, made);
        pen.controls.push((Touch::Color(spot.clone()), made));
        return area;
    }

    match value {
        Expression::Number(held) => {
            let span = span_of(*held);
            let made = tree.in_parent(area, |tree| tree.add_drag_value(0.0 - span, span, *held));
            fill(tree, made);
            pen.controls.push((Touch::Number(spot.clone()), made));
        }
        Expression::Truth(held) => {
            let made = tree.in_parent(area, |tree| tree.add_checkbox("", *held));
            fill(tree, made);
            pen.controls.push((Touch::Truth(spot.clone()), made));
        }
        Expression::Text(held) => {
            let made = tree.in_parent(area, |tree| tree.add_text_input_with_value("words", held));
            fill(tree, made);
            pen.controls.push((Touch::Words(spot.clone()), made));
        }
        Expression::Tuple(parts) if flat(parts) => {
            for (at, part) in parts.iter().enumerate() {
                let Expression::Number(one) = part else {
                    continue;
                };
                let span = span_of(*one);
                let made = tree.in_parent(area, |tree| tree.add_drag_value(0.0 - span, span, *one));
                fill(tree, made);
                pen.controls
                    .push((Touch::Number(spot.into(at as u16)), made));
            }
        }
        _ => {
            let made = tree.in_parent(area, |tree| {
                tree.add_dropdown(KINDS, data::edit::kind_of(value))
            });
            let held = skin(tree, made);
            hold(tree, held, KIND_WIDTH);
            pen.controls.push((Touch::Kind(spot.clone()), made));
            match value {
                Expression::Name(_) | Expression::Field { .. } => {
                    let now = write::value(value);
                    let names = placeable(program, &now);
                    let list: Vec<&str> = names.iter().map(|one| one.as_str()).collect();
                    let at = names.iter().position(|one| one == &now).unwrap_or(0);
                    let made = tree.in_parent(area, |tree| tree.add_dropdown_searchable(&list, at));
                    let held = skin(tree, made);
                    fill(tree, held);
                    pen.controls.push((Touch::Name(spot.clone()), made));
                }
                Expression::Binary { operator, .. } => {
                    let marks: Vec<&str> = OPERATORS.iter().map(|(mark, _)| *mark).collect();
                    let now = OPERATORS
                        .iter()
                        .position(|(_, held)| held == operator)
                        .unwrap_or(0);
                    let made = tree.in_parent(area, |tree| tree.add_dropdown(&marks, now));
                    let held = skin(tree, made);
                    fill(tree, held);
                    pen.controls.push((Touch::Operator(spot.clone()), made));
                }
                Expression::Call { name, .. } => {
                    let names = callable();
                    let at = names.iter().position(|one| one == name).unwrap_or(0);
                    let made = tree.in_parent(area, |tree| tree.add_dropdown(&names, at));
                    let held = skin(tree, made);
                    fill(tree, held);
                    pen.controls.push((Touch::Called(spot.clone()), made));
                }
                _ => {}
            }
            if depth < 3 {
                for at in 0..crate::queries::edit::parts(value) {
                    let mut held = value.clone();
                    if let Some(part) = edit::part_of(&mut held, at as u16) {
                        let part = part.clone();
                        worth(
                            tree,
                            pen,
                            home,
                            program,
                            "part",
                            &part,
                            &spot.into(at as u16),
                            depth + 1,
                        );
                    }
                }
            }
        }
    }
    area
}

fn span_of(held: f32) -> f32 {
    (held.abs().max(1.0) * 4.0).ceil()
}

fn flat(parts: &[Expression]) -> bool {
    (2..=4).contains(&parts.len())
        && parts
            .iter()
            .all(|part| matches!(part, Expression::Number(_)))
}

fn tint(value: &Expression) -> Option<[f32; 3]> {
    let Expression::Tuple(parts) = value else {
        return None;
    };
    if parts.len() != 3 {
        return None;
    }
    let mut held = [0.0; 3];
    let mut small = 0;
    for (at, part) in parts.iter().enumerate() {
        match part {
            Expression::Number(one) if (0.0..=1.0).contains(one) => {
                held[at] = *one;
                if *one < 1.0 {
                    small += 1;
                }
            }
            _ => return None,
        }
    }
    (small > 0).then_some(held)
}

fn skin(tree: &UiTreeBuilder, entity: Entity) -> Entity {
    tree.component::<nightshade::ecs::ChildOf>(entity)
        .map(|held| held.0)
        .unwrap_or(entity)
}

fn fill(tree: &mut UiTreeBuilder, entity: Entity) {
    tree.edit::<UiLayoutNode>(entity, |node| {
        node.flow_child_size = Some(Ab(Vec2::new(0.0, data::BUTTON_HEIGHT)).into());
        node.flex_grow = Some(1.0);
    });
}

fn hold(tree: &mut UiTreeBuilder, entity: Entity, width: f32) {
    tree.edit::<UiLayoutNode>(entity, move |node| {
        node.flow_child_size = Some(Ab(Vec2::new(width, data::BUTTON_HEIGHT)).into());
        node.flex_grow = Some(0.0);
    });
}

fn share(tree: &mut UiTreeBuilder, entity: Entity, part: f32) {
    tree.edit::<UiLayoutNode>(entity, move |node| {
        node.flow_child_size =
            Some(Rl(Vec2::new(part, 0.0)) + Ab(Vec2::new(0.0, data::BUTTON_HEIGHT)));
    });
}

fn section(tree: &mut UiTreeBuilder, home: Entity, label: &str, open: bool) -> Entity {
    let header = tree.in_parent(home, |tree| tree.add_collapsing_header(label, open));
    tree.component::<UiCollapsingHeaderData>(header)
        .map(|data| data.content_entity)
        .unwrap_or(header)
}

fn way(port: &Port) -> String {
    match port {
        Port::Reads(_) => "reads".to_string(),
        Port::Writes(_) => "writes".to_string(),
        Port::Enable(condition) => format!("enable {}", write::value(condition)),
        Port::Once => "once".to_string(),
        Port::Creates => "creates".to_string(),
        Port::Removes => "removes".to_string(),
        Port::Phase(name) => format!("phase {name}"),
        Port::Pure => "pure".to_string(),
        Port::Needs(_) => "needs".to_string(),
        Port::Provides(_) => "provides".to_string(),
        Port::Select(selection) => {
            let lead = match selection.cardinality {
                data::Cardinality::Every => "every",
                data::Cardinality::One => "one of",
                data::Cardinality::Set => "uses",
            };
            format!("{lead} {}", selection.label)
        }
    }
}

fn named(port: &Port) -> String {
    match port {
        Port::Reads(names) | Port::Writes(names) => names.first().cloned().unwrap_or_default(),
        Port::Select(selection) => selection
            .columns
            .first()
            .map(|column| column.name.clone())
            .unwrap_or_default(),
        _ => String::new(),
    }
}

fn knowable(program: &Program) -> Vec<String> {
    let mut held: Vec<String> = program.components.clone();
    held.extend(facility::columns().map(String::from));
    held.extend(facility::blocks().map(|block| block.name.to_string()));
    held.extend(program.configs.iter().map(|(name, _)| name.clone()));
    held.extend(program.resources.iter().map(|(name, _)| name.clone()));
    held.extend(program.events.iter().cloned());
    held
}

fn callable() -> Vec<&'static str> {
    facility::functions().collect()
}

fn spawnable(program: &Program) -> Vec<String> {
    let mut held: Vec<String> = facility::bindings()
        .chain(facility::columns())
        .map(String::from)
        .collect();
    held.extend(program.components.iter().cloned());
    held
}

fn placeable(program: &Program, now: &str) -> Vec<String> {
    let mut held = knowable(program);
    for (block, fields) in program.configs.iter().chain(program.resources.iter()) {
        for field in fields {
            held.push(format!("{block}.{}", field.name));
        }
    }
    for block in facility::blocks() {
        for field in block.fields {
            held.push(format!("{}.{field}", block.name));
        }
    }
    if !now.is_empty() && !held.iter().any(|one| one == now) {
        held.insert(0, now.to_string());
    }
    held
}

fn school(tree: &mut UiTreeBuilder, pen: &mut Pen, home: Entity, program: &Program, step: usize) {
    tree.in_parent(home, |tree| {
        tree.add_heading("Build breakout");
        for line in data::lesson::WELCOME {
            for part in fold(line, data::LINE_WIDTH) {
                chip(tree, &part, DIM);
            }
        }
        let row = tree.add_row();
        let made = tree.in_parent(row, |tree| {
            (
                tree.add_button("start from nothing"),
                tree.add_button("show me the answer"),
            )
        });
        share(tree, made.0, 49.0);
        share(tree, made.1, 49.0);
        pen.controls.push((Touch::Empty, made.0));
        pen.controls.push((Touch::Answer, made.1));
        tree.add_separator();
    });

    for (at, held) in data::lesson::STEPS.iter().enumerate() {
        let done = lesson::met(program, &held.want);
        let row = tree.in_parent(home, |tree| tree.add_row());
        let made = tree.in_parent(row, |tree| {
            let made = tree.add_button(&format!("{}. {}", at + 1, held.title));
            wide(tree, if done { "  done" } else { "" }, SHARED);
            made
        });
        share(tree, made, 74.0);
        pen.controls.push((Touch::Step(at), made));
        if step != at {
            continue;
        }
        tree.in_parent(home, |tree| {
            for line in held.note {
                for part in fold(line, data::LINE_WIDTH) {
                    chip(tree, &format!("   {part}"), CODE);
                }
            }
        });
    }
}

fn paper(tree: &mut UiTreeBuilder, program: &Program) {
    for line in write::program(program).split('\n') {
        for part in fold(line, data::LINE_WIDTH) {
            chip(tree, &part, CODE);
        }
    }
}

fn fold(line: &str, width: usize) -> Vec<String> {
    let mut parts = Vec::new();
    let lead: String = line.chars().take_while(|one| *one == ' ').collect();
    let mut held = lead.clone();
    for word in line.split_whitespace() {
        if held.trim().len() + word.len() + 1 > width && !held.trim().is_empty() {
            parts.push(std::mem::take(&mut held));
            held = format!("{lead}   ");
        }
        if !held.trim().is_empty() {
            held.push(' ');
        }
        held.push_str(word);
    }
    if !held.trim().is_empty() || parts.is_empty() {
        parts.push(held);
    }
    parts
}

fn wide(tree: &mut UiTreeBuilder, text: &str, tone: [f32; 4]) {
    tree.label_node(text, data::CHIP_SIZE)
        .flow_child(Ab(Vec2::new(0.0, data::BUTTON_HEIGHT)))
        .flex_grow(1.0)
        .color_raw::<UiBase>(Vec4::new(tone[0], tone[1], tone[2], tone[3]))
        .entity();
}

fn chip(tree: &mut UiTreeBuilder, text: &str, tone: [f32; 4]) {
    tree.label_node(text, data::CHIP_SIZE)
        .color_raw::<UiBase>(Vec4::new(tone[0], tone[1], tone[2], tone[3]))
        .entity();
}

pub fn freshen(
    studio: Res<Studio>,
    script: Res<Script>,
    store: Res<Store>,
    mut canvases: Query<(&mut UiCanvasData,)>,
) {
    let Some(canvas) = studio.graph else {
        return;
    };
    let cards = shape::cards(&script.plan, &store);
    let (_, paint) = crate::systems::inspect::plot(&script.plan, &cards, studio.chosen);
    canvases.for_entity(canvas, |(data,)| {
        data.commands = paint.clone();
    });
}

pub fn listen(frame: Res<RetainedUiFrame>, mut studio: ResMut<Studio>, mut script: ResMut<Script>) {
    let mut redraw = false;
    for event in ui_events(&frame) {
        let entity = match event {
            UiEvent::ButtonClicked(entity)
            | UiEvent::SliderChanged { entity, .. }
            | UiEvent::DragValueChanged { entity, .. }
            | UiEvent::CheckboxChanged { entity, .. }
            | UiEvent::ToggleChanged { entity, .. }
            | UiEvent::ColorPickerChanged { entity, .. }
            | UiEvent::DropdownChanged { entity, .. }
            | UiEvent::TabChanged { entity, .. }
            | UiEvent::TextInputChanged { entity, .. }
            | UiEvent::TextInputSubmitted { entity, .. }
            | UiEvent::CanvasClicked { entity, .. } => *entity,
            _ => continue,
        };
        if studio.input == Some(entity)
            && let UiEvent::TextInputChanged { text, .. } | UiEvent::TextInputSubmitted { text, .. } =
                event
        {
            studio.name = text.clone();
            continue;
        }
        let Some(touch) = studio
            .controls
            .iter()
            .find(|(_, held)| held == &entity)
            .map(|(touch, _)| touch.clone())
        else {
            continue;
        };
        let name = studio.name.clone();
        let form = studio.form;
        let held = &mut script.source;
        let mut shaped = true;
        let again = !matches!(
            touch,
            Touch::Number(_)
                | Touch::Color(_)
                | Touch::Truth(_)
                | Touch::Words(_)
                | Touch::PortKind(_)
                | Touch::NewForm
        );
        match (&touch, event) {
            (Touch::Graph, UiEvent::CanvasClicked { command_id, .. }) => {
                let at = (*command_id as usize).saturating_sub(crate::systems::inspect::MARK);
                studio.chosen = if studio.chosen == Some(at) {
                    None
                } else {
                    Some(at)
                };
                shaped = false;
            }
            (Touch::Tab, UiEvent::TabChanged { tab_index, .. }) => {
                studio.tab = *tab_index;
                shaped = false;
            }
            (Touch::Number(spot), UiEvent::DragValueChanged { value, .. })
            | (Touch::Number(spot), UiEvent::SliderChanged { value, .. }) => {
                edit::set_number(held, spot, *value)
            }
            (Touch::Truth(spot), UiEvent::CheckboxChanged { value, .. })
            | (Touch::Truth(spot), UiEvent::ToggleChanged { value, .. }) => {
                edit::set_truth(held, spot, *value)
            }
            (Touch::Color(spot), UiEvent::ColorPickerChanged { color, .. }) => {
                edit::set_color(held, spot, [color.x, color.y, color.z])
            }
            (Touch::Words(spot), UiEvent::TextInputChanged { text, .. }) => {
                edit::set_words(held, spot, text)
            }
            (Touch::Kind(spot), UiEvent::DropdownChanged { selected_index, .. }) => {
                edit::set_kind(held, spot, *selected_index)
            }
            (Touch::Operator(spot), UiEvent::DropdownChanged { selected_index, .. }) => {
                edit::set_operator(held, spot, *selected_index)
            }
            (Touch::Called(spot), UiEvent::DropdownChanged { selected_index, .. }) => {
                if let Some(wanted) = callable().get(*selected_index) {
                    edit::set_call(held, spot, wanted)
                }
            }
            (Touch::Name(spot), UiEvent::DropdownChanged { selected_index, .. }) => {
                let names = placeable(held, "");
                if let Some(wanted) = names.get(*selected_index) {
                    let wanted = wanted.clone();
                    edit::set_name(held, spot, &wanted);
                }
            }
            (Touch::Place(system, trail), UiEvent::DropdownChanged { selected_index, .. }) => {
                let names = placeable(held, "");
                if let Some(wanted) = names.get(*selected_index) {
                    let wanted = wanted.clone();
                    edit::set_place(held, *system, trail, &wanted);
                }
            }
            (
                Touch::Binding(system, trail, at),
                UiEvent::DropdownChanged { selected_index, .. },
            ) => {
                let names = spawnable(held);
                if let Some(wanted) = names.get(*selected_index) {
                    let wanted = wanted.clone();
                    edit::set_binding(held, *system, trail, *at, &wanted);
                }
            }
            (Touch::PortName(system, port), UiEvent::DropdownChanged { selected_index, .. }) => {
                let names = knowable(held);
                if let Some(wanted) = names.get(*selected_index) {
                    let wanted = wanted.clone();
                    edit::set_port_name(held, *system, *port, &wanted);
                }
            }
            (Touch::PortKind(_), UiEvent::DropdownChanged { selected_index, .. }) => {
                studio.port = *selected_index;
                shaped = false;
            }
            (Touch::NewForm, UiEvent::DropdownChanged { selected_index, .. }) => {
                studio.form = *selected_index;
                shaped = false;
            }
            (Touch::AddComponent, UiEvent::ButtonClicked(_)) => edit::add_component(held, &name),
            (Touch::DropComponent(at), UiEvent::ButtonClicked(_)) => {
                edit::drop_component(held, *at)
            }
            (Touch::AddField(block, path), UiEvent::ButtonClicked(_)) => {
                edit::add_field(held, *block, path, &name, false)
            }
            (Touch::AddGroup(block, path), UiEvent::ButtonClicked(_)) => {
                edit::add_field(held, *block, path, &name, true)
            }
            (Touch::DropField(block, at), UiEvent::ButtonClicked(_)) => {
                edit::drop_field(held, *block, *at)
            }
            (Touch::AddSystem, UiEvent::ButtonClicked(_)) => edit::add_system(held, &name),
            (Touch::DropSystem(at), UiEvent::ButtonClicked(_)) => edit::drop_system(held, *at),
            (Touch::PickSystem(at), UiEvent::ButtonClicked(_)) => {
                studio.chosen = if studio.chosen == Some(*at) {
                    None
                } else {
                    Some(*at)
                };
                shaped = false;
            }
            (Touch::AddPort(system, _), UiEvent::ButtonClicked(_)) => {
                let kind = studio.port;
                edit::add_port(held, *system, kind)
            }
            (Touch::DropPort(system, port), UiEvent::ButtonClicked(_)) => {
                edit::drop_port(held, *system, *port)
            }
            (Touch::AddStatement(system, trail, _), UiEvent::ButtonClicked(_)) => {
                edit::add_inside(held, *system, trail, form)
            }
            (Touch::DropStatement(system, trail), UiEvent::ButtonClicked(_)) => {
                edit::drop_statement(held, *system, trail)
            }
            (Touch::LiftStatement(system, trail, down), UiEvent::ButtonClicked(_)) => {
                edit::lift_statement(held, *system, trail, *down)
            }
            (Touch::AddBinding(system, trail), UiEvent::ButtonClicked(_)) => {
                edit::add_binding(held, *system, trail)
            }
            (Touch::Step(at), UiEvent::ButtonClicked(_)) => {
                studio.step = *at;
                shaped = false;
            }
            (Touch::Empty, UiEvent::ButtonClicked(_)) => {
                script.wanted = Some(data::lesson::EMPTY.to_string());
                shaped = false;
            }
            (Touch::Answer, UiEvent::ButtonClicked(_)) => {
                let wanted = data::lesson::STEPS
                    .get(studio.step)
                    .map(|step| step.answer.to_string());
                script.wanted = wanted;
                shaped = false;
            }
            _ => shaped = false,
        }
        if shaped {
            script.dirty = true;
        }
        redraw = redraw || again;
    }
    if redraw {
        studio.stale = true;
    }
}
