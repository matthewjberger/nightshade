use crate::commands::interface::set_fill;
use crate::components::{Ammo, Combo, Holding};
use crate::data;
use crate::resources::{Interface, Mode, Ring};
use nalgebra_glm::Vec4;
use nightshade::prelude::*;
use nightshade_ui::prelude::{UiBase, ui_set_color, ui_set_text, ui_set_visible};
use nightshade_ui::resources::RetainedUiDirty;

pub fn show(
    ring: Res<Ring>,
    mut handles: ResMut<Interface>,
    mut dirty: ResMut<RetainedUiDirty>,
    mut commands: Commands,
    heroes: Query<(&Holding, &Ammo, &Combo)>,
) {
    if handles.hero_fill == Entity::default() {
        return;
    }
    let inner = data::BAR_WIDTH - 6.0;
    let hero = (ring.hero_health / data::HERO_HEALTH).clamp(0.0, 1.0);
    if (hero - handles.last_hero).abs() > 0.001 {
        handles.last_hero = hero;
        set_fill(&mut dirty, &mut commands, handles.hero_fill, inner * hero);
        let color = if hero <= data::LOW_HEALTH {
            data::HERO_BAR_LOW
        } else {
            data::HERO_BAR_COLOR
        };
        ui_set_color::<UiBase>(&mut commands, handles.hero_fill, colour(color));
    }

    let chief = ring.chief != Entity::default();
    let (thug, name) = if chief {
        (ring.chief_share, ring.chief_name)
    } else {
        (ring.mark, ring.mark_name)
    };
    let thug = thug.clamp(0.0, 1.0);
    let wanted = (ring.alive > 0 || chief) && !ring.cleared;
    if wanted != handles.thug_shown {
        handles.thug_shown = wanted;
        ui_set_visible(&mut dirty, &mut commands, handles.thug_panel, wanted);
    }
    if chief != handles.chief_shown {
        handles.chief_shown = chief;
        let color = if chief {
            data::BOSS_BAR_COLOR
        } else {
            data::THUG_BAR_COLOR
        };
        ui_set_color::<UiBase>(&mut commands, handles.thug_fill, colour(color));
    }
    if wanted && (thug - handles.last_thug).abs() > 0.001 {
        handles.last_thug = thug;
        set_fill(&mut dirty, &mut commands, handles.thug_fill, inner * thug);
    }
    if wanted && handles.shown_name != name {
        handles.shown_name = name;
        ui_set_text(&mut commands, handles.thug_name, name);
    }

    if ring.score != handles.last_score || !handles.ready {
        handles.ready = true;
        handles.last_score = ring.score;
        ui_set_text(&mut commands, handles.score, &format!("{:07}", ring.score));
    }

    let title = match ring.mode {
        Mode::Story => format!(
            "STAGE {}  {}   WAVE {}/{}",
            ring.stage + 1,
            data::STAGES[ring.stage.min(data::STAGE_COUNT - 1)].name,
            ring.wave + 1,
            ring.waves.max(1)
        ),
        Mode::Arcade => format!("ARCADE  ROUND {}", ring.round + 1),
    };
    if title != handles.shown_stage {
        handles.shown_stage = title.clone();
        ui_set_text(&mut commands, handles.stage, &title);
    }

    let mut carried = String::new();
    heroes.for_each(|_entity, (holding, ammo, _)| {
        if holding.0 == 0 {
            return;
        }
        let plan = &data::WEAPON_KINDS[(holding.0 - 1) as usize];
        carried = format!("{}  x{}", plan.name, ammo.0);
    });
    if carried != handles.shown_weapon {
        handles.shown_weapon = carried.clone();
        ui_set_text(&mut commands, handles.weapon, &carried);
    }

    let blink = ring.flash > 0.0 && ((ring.flash / data::GO_BLINK) as u32).is_multiple_of(2);
    if blink != handles.go_shown {
        handles.go_shown = blink;
        ui_set_visible(&mut dirty, &mut commands, handles.go, blink);
    }
}

fn colour(value: [f32; 4]) -> Vec4 {
    Vec4::new(value[0], value[1], value[2], value[3])
}
