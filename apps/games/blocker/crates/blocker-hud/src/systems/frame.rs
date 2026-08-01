use crate::commands::{paint_slot, set_border};
use crate::data::settings::Settings;
use crate::queries::backdrop_wanted;
use crate::resources::{ChatPanel, HudPanels, ShellHandles};
use crate::systems::hud::rate::show_the_rate;
use crate::theme;
use blocker_blocks::data::blocks::BlockKind;
use blocker_figure::queries::read_the_hero;
use blocker_figure::resources::Hero;
use blocker_player::components::{Motion, WatchSpeed};
use blocker_player::resources::{Aim, Prompt};
use blocker_pocket::queries::{selected_stack, stack_at};
use blocker_pocket::resources::Inventory;
use blocker_screen::data::screen::Screen;
use nightshade::prelude::*;
use nightshade_config::prelude::Config;
use nightshade_platform::prelude::Time;
use nightshade_render::data::config::RendererState;
use nightshade_state::prelude::*;
use nightshade_toast::queries::showing;
use nightshade_ui::components::UiNodeInteraction;
use nightshade_ui::prelude::*;
use nightshade_ui::resources::RetainedUiDirty;

pub fn calm_the_interface(
    chat_panel: Res<ChatPanel>,
    mut nav: ResMut<RetainedUiGamepadNav>,
    mut interaction: ResMut<RetainedUiInteraction>,
    mut commands: Commands,
) {
    nav.enabled = false;
    interaction.focused_entity = None;
    for tab in chat_panel.tabs.iter().copied() {
        commands.queue(move |storage, _| {
            if let Some(interaction) = storage.get_mut::<UiNodeInteraction>(tab) {
                interaction.tab_index = None;
            }
        });
    }
}

pub fn show_the_backdrop(
    state: Res<State<Screen>>,
    mut shell: ResMut<ShellHandles>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
) {
    let screen = state.current;
    if shell.root == Entity::default() || shell.shown == Some(screen) {
        return;
    }
    shell.shown = Some(screen);
    ui_set_visible(
        &mut dirty,
        &mut commands,
        shell.backdrop,
        backdrop_wanted(screen),
    );
}

pub fn refresh(
    hud: HudPanels,
    inventory: Res<Inventory>,
    aim: Res<Aim>,
    prompt: Res<Prompt>,
    config: Res<Config<Settings>>,
    time: Res<Time>,
    renderer_state: Res<RendererState>,
    hero: Res<Hero>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    mut bodies: Query<(&Motion, &WatchSpeed)>,
) {
    if hud.shell.root == Entity::default() {
        return;
    }
    let screen = hud.shell.shown.unwrap_or_default();
    show_the_rate(
        hud.rate,
        config.toggle(Settings::ShowRate),
        &time,
        renderer_state.gpu_frame_time_ms,
        &mut dirty,
        &mut commands,
    );
    show_the_prompt(hud.prompt, &prompt, screen, &mut dirty, &mut commands);
    let shown = showing(hud.toast);
    if shown && hud.toast_panel.shown != hud.toast.line {
        hud.toast_panel.shown = hud.toast.line.clone();
        ui_set_text(&mut commands, hud.toast_panel.label, &hud.toast.line);
    }
    if hud.toast_panel.visible != shown {
        hud.toast_panel.visible = shown;
        ui_set_visible(&mut dirty, &mut commands, hud.toast_panel.panel, shown);
    }
    let Some((motion, watch_speed)) =
        read_the_hero(&hero, &mut bodies, |(motion, watch)| (*motion, watch.0))
    else {
        return;
    };
    let watching = motion == Motion::Watch;
    if hud.hotbar.shown_watching != Some(watching) {
        hud.hotbar.shown_watching = Some(watching);
        ui_set_visible(&mut dirty, &mut commands, hud.hotbar.gear, !watching);
    }
    for index in 0..blocker_blocks::data::items::HOTBAR_SLOTS {
        let stack = stack_at(&inventory.slots, index);
        let slot = hud.hotbar.slots[index];
        if hud.hotbar.shown[index] != stack {
            let was = hud.hotbar.shown[index];
            hud.hotbar.shown[index] = stack;
            paint_slot(hud.layers, &slot, stack, was, &mut dirty, &mut commands);
        }
    }

    if hud.hotbar.shown_selected != Some(inventory.selected.0) {
        if let Some(previous) = hud.hotbar.shown_selected {
            let frame = hud.hotbar.slots[previous].frame;
            ui_set_color::<UiBase>(&mut commands, frame, theme::SLOT_COLOR);
            set_border(&mut commands, frame, theme::SLOT_BORDER);
        }
        let frame = hud.hotbar.slots[inventory.selected.0].frame;
        ui_set_color::<UiBase>(&mut commands, frame, theme::SLOT_SELECTED_COLOR);
        set_border(&mut commands, frame, theme::SLOT_SELECTED_BORDER);
        hud.hotbar.shown_selected = Some(inventory.selected.0);
    }

    let held = selected_stack(&inventory)
        .map(|stack| stack.item.name())
        .unwrap_or("Empty hand");
    let looking = aim
        .0
        .map(|hit| {
            format!(
                "   looking at {}",
                BlockKind::from_index(hit.kind).info().name
            )
        })
        .unwrap_or_default();
    let mode = if motion == Motion::Fly {
        "   flying"
    } else {
        ""
    };
    let status = if motion == Motion::Watch {
        let speed = watch_speed.round() as i32;
        format!("watching   {speed} blocks a second")
    } else {
        format!("{held}{looking}{mode}")
    };
    if status != hud.hotbar.shown_status {
        ui_set_text(&mut commands, hud.hotbar.status, &status);
        hud.hotbar.shown_status = status;
    }
}

fn show_the_prompt(
    panel: &mut crate::resources::PromptPanel,
    prompt: &Prompt,
    screen: Screen,
    dirty: &mut RetainedUiDirty,
    commands: &mut Commands,
) {
    let prompt_text = match prompt {
        Prompt::None => None,
        Prompt::Harvest(_) => Some("E  Pick".to_string()),
        Prompt::Door(_, open) => Some(if *open {
            "E  Close the door".to_string()
        } else {
            "E  Open the door".to_string()
        }),
        Prompt::Craft => Some("E  Craft".to_string()),
        Prompt::Chest(_) => Some("E  Open the chest".to_string()),
        Prompt::Board(_) => Some("E  Get in the boat".to_string()),
        Prompt::Leave => Some("E  Step out of the boat".to_string()),
        Prompt::Sign(_) => Some("E  Read".to_string()),
        Prompt::Talk(name) => Some(format!("E  Talk to {name}")),
    };
    let wanted = if screen == Screen::Playing {
        prompt_text
    } else {
        None
    };
    if wanted == panel.shown {
        return;
    }
    ui_set_visible(dirty, commands, panel.panel, wanted.is_some());
    if let Some(text) = &wanted {
        ui_set_text(commands, panel.label, text);
    }
    panel.shown = wanted;
}
