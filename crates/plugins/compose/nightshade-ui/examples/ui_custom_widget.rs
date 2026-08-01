use nalgebra_glm::{Vec2, Vec4, vec2, vec4};
use nightshade::prelude::nalgebra_glm;
use nightshade_ui::prelude::*;
use nightshade_ui::{UiSystemPhase, insert_ui_resources, ui_systems_with};

const SWATCH: f32 = 40.0;
const GAP: f32 = 6.0;

#[derive(Default)]
struct Swatches {
    canvas: Option<Entity>,
    colors: Vec<Vec4>,
    selected: usize,
}

fn host(width: u32, height: u32) -> nightshade::ecs::World {
    let mut ecs = nightshade::ecs::World::new();
    insert_ui_resources(&mut ecs.resources);
    let window = ecs.res_mut::<nightshade_platform::contract::window::Window>();
    window.cached_viewport_size = Some((width, height));
    window.cached_scale_factor = 1.0;
    ecs.res_mut::<RetainedUiRuntime>().visible = true;
    ecs
}

fn swatch_interaction(
    mut swatches: nightshade::ecs::system_param::ResMut<Swatches>,
    input: nightshade::ecs::system_param::Res<nightshade_platform::contract::input::Input>,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(canvas) = swatches.canvas else {
        return;
    };
    if !storage
        .get::<UiNodeInteraction>(canvas)
        .is_some_and(|interaction| interaction.clicked)
    {
        return;
    }
    let pointer = input.mouse.position;
    let origin = storage
        .get::<UiLayoutNode>(canvas)
        .map(|node| node.computed_rect.min)
        .unwrap_or_default();
    if let Some(command) = ui_canvas_hit_test(storage, canvas, pointer - origin) {
        swatches.selected = command as usize;
    }
}

fn swatch_draw(
    swatches: nightshade::ecs::system_param::Res<Swatches>,
    storage: &mut nightshade::ecs::Storage,
) {
    let (Some(canvas), colors, selected) =
        (swatches.canvas, swatches.colors.clone(), swatches.selected)
    else {
        return;
    };

    ui_canvas_clear(storage, canvas);
    for (index, color) in colors.iter().enumerate() {
        let x = index as f32 * (SWATCH + GAP);
        ui_canvas_rect(
            storage,
            canvas,
            vec2(x, 0.0),
            vec2(SWATCH, SWATCH),
            *color,
            4.0,
        );
        ui_canvas_tag(storage, canvas, index as u32);
        if index == selected {
            ui_canvas_rect(
                storage,
                canvas,
                vec2(x, SWATCH + 2.0),
                vec2(SWATCH, 3.0),
                vec4(1.0, 1.0, 1.0, 1.0),
                1.5,
            );
        }
    }
}

fn main() {
    let mut ecs = host(1280, 720);
    ecs.insert_resource(Swatches {
        canvas: None,
        colors: vec![
            vec4(0.9, 0.3, 0.3, 1.0),
            vec4(0.3, 0.8, 0.4, 1.0),
            vec4(0.3, 0.5, 0.9, 1.0),
            vec4(0.9, 0.8, 0.3, 1.0),
        ],
        selected: 0,
    });

    let strip_width = 4.0 * SWATCH + 3.0 * GAP;
    {
        ecs.with_commands(|ecs, commands| {
            let mut tree = UiTreeBuilder::new(nightshade_ui::commands::builder::ui_build_context(
                &mut ecs.resources,
                commands,
            ));
            let root = tree
                .add_node()
                .boundary(Rl(vec2(0.0, 0.0)), Rl(vec2(100.0, 100.0)))
                .entity();
            let panel = tree.in_parent(root, |tree| {
                tree.add_node()
                    .window(
                        Ab(vec2(24.0, 24.0)),
                        Ab(vec2(strip_width + 32.0, 120.0)),
                        Anchor::TopLeft,
                    )
                    .card()
                    .flow(FlowDirection::Vertical, 16.0, 8.0)
                    .entity()
            });
            let canvas = tree.in_parent(panel, |tree| {
                tree.add_label("Accent");
                tree.add_canvas_interactive(Vec2::new(strip_width, SWATCH + 6.0))
            });
            tree.finish();
            ecs.res_mut::<Swatches>().canvas = Some(canvas);
        });
    }

    let mut systems = ui_systems_with(vec![
        (
            UiSystemPhase::Interaction,
            nightshade_ui::ui_system(swatch_interaction),
        ),
        (
            UiSystemPhase::PreDraw,
            nightshade_ui::ui_system(swatch_draw),
        ),
    ]);

    ecs.resources
        .res_mut::<nightshade::ecs::HierarchyIndex>()
        .sync(&mut ecs.storage);
    for system in &mut systems {
        system(&mut ecs);
    }

    println!("selected swatch: {}", ecs.res::<Swatches>().selected);
    println!(
        "draw list: {} rects",
        ecs.res::<RetainedUiFrame>().rects.len()
    );

    ecs.res_mut::<Swatches>().selected = 2;
    for system in &mut systems {
        system(&mut ecs);
    }
    println!("after select: {}", ecs.res::<Swatches>().selected);
}
