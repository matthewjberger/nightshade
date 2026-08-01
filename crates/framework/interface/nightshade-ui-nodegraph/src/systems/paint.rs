use crate::commands::ink_a_wire;
use crate::components::{
    NodeField, NodeInletNames, NodeInlets, NodeNote, NodeOutletNames, NodeOutlets, NodePlace,
    NodeTint, NodeTitle, WireFrom, WireTo,
};
use crate::data::{Holding, Socket};
use crate::queries::{
    inlet_of, inlet_place, node_span, note_place, note_rect, outlet_of, outlet_place,
};
use crate::resources::{GraphGrip, GraphView, NodeGraphSettings};
use nalgebra_glm::{Vec2, Vec4, vec2};
use nightshade::ecs::system_param::Res;
use nightshade::ecs::{Entity, Storage};
use nightshade_ui::prelude::{
    UiLayoutNode, ui_canvas_circle, ui_canvas_clear, ui_canvas_line, ui_canvas_rect,
    ui_canvas_rect_stroke, ui_canvas_tag, ui_canvas_text,
};

struct Standing {
    node: Entity,
    place: Vec2,
    inlets: u32,
    outlets: u32,
    tint: Vec4,
    title: String,
    inlet_names: Vec<String>,
    outlet_names: Vec<String>,
    note: Option<String>,
    fielded: bool,
}

pub fn paint_the_graph(
    settings: Res<NodeGraphSettings>,
    view: Res<GraphView>,
    grip: Res<GraphGrip>,
    storage: &mut Storage,
) {
    let Some(canvas) = view.canvas else {
        return;
    };
    let span = storage
        .get::<UiLayoutNode>(canvas)
        .map(|node| node.computed_rect.size())
        .unwrap_or_default();

    let (standing, strung) = {
        let rows = &*storage;
        let standing: Vec<Standing> = rows
            .query_ref::<(
                &NodePlace,
                &NodeInlets,
                &NodeOutlets,
                &NodeTint,
                &NodeTitle,
                Option<&NodeInletNames>,
                Option<&NodeOutletNames>,
                Option<&NodeNote>,
            )>()
            .iter()
            .map(
                |(node, (place, inlets, outlets, tint, title, inlet_names, outlet_names, note))| {
                    Standing {
                        node,
                        place: place.0,
                        inlets: inlets.0,
                        outlets: outlets.0,
                        tint: tint.0,
                        title: title.0.clone(),
                        inlet_names: inlet_names.map(|names| names.0.clone()).unwrap_or_default(),
                        outlet_names: outlet_names
                            .map(|names| names.0.clone())
                            .unwrap_or_default(),
                        note: note.map(|note| note.0.clone()),
                        fielded: rows.get::<NodeField>(node).is_some_and(|field| field.0),
                    }
                },
            )
            .collect();
        let strung: Vec<(Vec2, Vec2)> = rows
            .query_ref::<(&WireFrom, &WireTo)>()
            .iter()
            .filter_map(|(_wire, (from, to))| {
                let tail = outlet_of(rows, &settings, from.0)?;
                let head = inlet_of(rows, &settings, to.0)?;
                Some((tail, head))
            })
            .collect();
        (standing, strung)
    };

    ui_canvas_clear(storage, canvas);
    paint_the_grid(storage, canvas, &settings, view.pan, span);

    for (tail, head) in strung {
        ink_a_wire(
            storage,
            canvas,
            tail + view.pan,
            head + view.pan,
            &settings,
            settings.wire_tint,
        );
    }

    paint_the_dangling(storage, canvas, &settings, &view, &grip);

    for body in &standing {
        paint_a_node(storage, canvas, &settings, view.pan, &grip, body);
    }
}

fn paint_the_grid(
    storage: &mut Storage,
    canvas: Entity,
    settings: &NodeGraphSettings,
    pan: Vec2,
    span: Vec2,
) {
    if settings.grid_spacing <= 0.0 || span.x <= 0.0 || span.y <= 0.0 {
        return;
    }
    let mut across = pan.x.rem_euclid(settings.grid_spacing);
    while across < span.x {
        ui_canvas_line(
            storage,
            canvas,
            vec2(across, 0.0),
            vec2(across, span.y),
            settings.grid_width,
            settings.grid_tint,
        );
        across += settings.grid_spacing;
    }
    let mut down = pan.y.rem_euclid(settings.grid_spacing);
    while down < span.y {
        ui_canvas_line(
            storage,
            canvas,
            vec2(0.0, down),
            vec2(span.x, down),
            settings.grid_width,
            settings.grid_tint,
        );
        down += settings.grid_spacing;
    }
}

fn paint_the_dangling(
    storage: &mut Storage,
    canvas: Entity,
    settings: &NodeGraphSettings,
    view: &GraphView,
    grip: &GraphGrip,
) {
    let ends = match grip.holding {
        Holding::Outlet(socket) => {
            outlet_of(storage, settings, socket).map(|tail| (tail, grip.pointer))
        }
        Holding::Inlet(socket) => {
            inlet_of(storage, settings, socket).map(|head| (grip.pointer, head))
        }
        Holding::Nothing | Holding::View | Holding::Node(_) | Holding::Note(_) => None,
    };
    let Some((tail, head)) = ends else {
        return;
    };
    ink_a_wire(
        storage,
        canvas,
        tail + view.pan,
        head + view.pan,
        settings,
        settings.dangling_tint,
    );
}

fn paint_a_node(
    storage: &mut Storage,
    canvas: Entity,
    settings: &NodeGraphSettings,
    pan: Vec2,
    grip: &GraphGrip,
    body: &Standing,
) {
    let held = matches!(grip.holding, Holding::Node(node) if node == body.node)
        || matches!(grip.over, Holding::Node(node) if node == body.node);
    let noted = matches!(grip.holding, Holding::Note(node) if node == body.node)
        || matches!(grip.over, Holding::Note(node) if node == body.node);
    let corner = pan + body.place;
    let span = node_span(body.inlets, body.outlets, body.note.is_some(), settings);

    ui_canvas_rect(
        storage,
        canvas,
        corner,
        span,
        settings.body_tint,
        settings.corner,
    );
    ui_canvas_tag(storage, canvas, body.node.id);
    ui_canvas_rect(
        storage,
        canvas,
        corner,
        vec2(span.x, settings.header_height),
        body.tint,
        settings.corner,
    );
    ui_canvas_rect_stroke(
        storage,
        canvas,
        corner,
        span,
        if held {
            settings.held_tint
        } else {
            settings.edge_tint
        },
        settings.edge_width,
        settings.corner,
    );
    ui_canvas_text(
        storage,
        canvas,
        &body.title,
        corner + settings.title_inset,
        settings.title_size,
        settings.title_tint,
    );

    for index in 0..body.outlets {
        let at = pan + outlet_place(body.place, index, settings);
        paint_a_socket(
            storage,
            canvas,
            settings,
            grip,
            at,
            Holding::Outlet(Socket {
                node: body.node,
                index,
            }),
        );
        if let Some(name) = body.outlet_names.get(index as usize) {
            let width = name.chars().count() as f32 * settings.label_size * settings.label_advance;
            paint_a_label(
                storage,
                canvas,
                settings,
                name,
                vec2(at.x - settings.label_inset - width, at.y),
            );
        }
    }

    for index in 0..body.inlets {
        let at = pan + inlet_place(body.place, index, body.outlets, settings);
        paint_a_socket(
            storage,
            canvas,
            settings,
            grip,
            at,
            Holding::Inlet(Socket {
                node: body.node,
                index,
            }),
        );
        if let Some(name) = body.inlet_names.get(index as usize) {
            paint_a_label(
                storage,
                canvas,
                settings,
                name,
                vec2(at.x + settings.label_inset, at.y),
            );
        }
    }

    if let Some(note) = body.note.as_ref() {
        if body.fielded {
            let row = note_rect(body.place, body.inlets, body.outlets, settings);
            ui_canvas_rect(
                storage,
                canvas,
                pan + row.min,
                row.size(),
                if noted {
                    settings.note_lit_tint
                } else {
                    settings.note_field_tint
                },
                settings.corner * 0.5,
            );
            if noted {
                paint_the_grips(storage, canvas, settings, pan + row.min, row.size());
            }
        }
        let at = pan + note_place(body.place, body.inlets, body.outlets, settings);
        ui_canvas_text(
            storage,
            canvas,
            note,
            at + vec2(0.0, settings.note_height * 0.25),
            settings.label_size,
            if noted {
                settings.lit_tint
            } else {
                settings.note_tint
            },
        );
    }
}

fn paint_the_grips(
    storage: &mut Storage,
    canvas: Entity,
    settings: &NodeGraphSettings,
    corner: Vec2,
    span: Vec2,
) {
    let lift = (span.y - settings.label_size) * 0.5;
    ui_canvas_text(
        storage,
        canvas,
        "<",
        vec2(corner.x + settings.grip_inset, corner.y + lift),
        settings.label_size,
        settings.lit_tint,
    );
    ui_canvas_text(
        storage,
        canvas,
        ">",
        vec2(
            corner.x + span.x - settings.grip_inset * 2.0,
            corner.y + lift,
        ),
        settings.label_size,
        settings.lit_tint,
    );
}

fn paint_a_label(
    storage: &mut Storage,
    canvas: Entity,
    settings: &NodeGraphSettings,
    name: &str,
    at: Vec2,
) {
    ui_canvas_text(
        storage,
        canvas,
        name,
        vec2(at.x, at.y - settings.label_size * 0.62),
        settings.label_size,
        settings.label_tint,
    );
}

fn paint_a_socket(
    storage: &mut Storage,
    canvas: Entity,
    settings: &NodeGraphSettings,
    grip: &GraphGrip,
    at: Vec2,
    which: Holding,
) {
    let lit = grip.over == which || grip.holding == which;
    ui_canvas_circle(
        storage,
        canvas,
        at,
        if lit {
            settings.socket_radius * 1.3
        } else {
            settings.socket_radius
        },
        if lit {
            settings.lit_tint
        } else {
            settings.socket_tint
        },
    );
}
