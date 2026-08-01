use crate::commands::camp::{apply_alone, choose_row, close_the_day, enter_the_room};
use crate::commands::edit_fighter;
use crate::commands::framing::{frame_the_talk, release_the_frame};
use crate::commands::take_the_upgrade;
use crate::commands::{
    add_experience, finish_the_speech, lift_morale, look_at, raise_bond, set_camp_phase, speak,
    spend, story,
};
use crate::components::{Called, Room, Stride};
use crate::data::{ACTIVITIES, Activity, Ask, COMPANIONS, Facing};
use crate::queries::camp::{deploy_limit, row_flags, upgrade_options};
use crate::queries::field::walkable;
use crate::queries::{direction, fighter_index, menu_step, mustered, mustering};
use crate::resources::{CameraRig, Camp, CampPhase, Campaign, Prompt, Terrain, UiHandles};
use crate::theme;
use nalgebra_glm::{Vec2, Vec3};
use nightshade::prelude::*;
use nightshade_input::Actions;
use nightshade_platform::prelude::*;
use nightshade_scene::prelude::*;
use nightshade_sprite::prelude::*;

pub fn read_the_camp(actions: Res<Actions<Ask>>, mut camp: ResMut<Camp>) {
    let (column, row) = direction(&actions);
    camp.direction = Vec2::new(column as f32, row as f32);
    camp.confirm = actions.just_pressed(Ask::Confirm);
    camp.cancel = actions.just_pressed(Ask::Cancel);
    camp.step = menu_step(&actions);
}

pub fn walk_the_commander(
    time: Res<Time>,
    terrain: Res<Terrain>,
    mut rig: ResMut<CameraRig>,
    mut camp: ResMut<Camp>,
    campaign: Res<Campaign>,
    leaders: Query<(&mut Stride, &mut Facing, &mut LocalTransform, &mut Sprite)>,
) {
    let roaming = campaign.phase == CampPhase::Roam;
    let wanted = if roaming {
        camp.direction
    } else {
        Vec2::zeros()
    };
    let moving = wanted.magnitude_squared() > 0.05;
    let forward = Vec2::new(-rig.yaw.sin(), -rig.yaw.cos());
    let right = crate::queries::rig::right(&rig);
    let heading = if moving {
        (right * wanted.x + forward * wanted.y).normalize()
    } else {
        Vec2::zeros()
    };
    let delta_time = time.delta_time;
    let mut feet = camp.feet;

    leaders.for_each(|_entity, (stride, facing, transform, sprite)| {
        let mut ground = Vec2::new(transform.translation.x, transform.translation.z);
        if moving {
            let step = heading * theme::WALK_PACE * delta_time;
            let along_x = Vec2::new(ground.x + step.x, ground.y);
            if walkable(&terrain, along_x) {
                ground = along_x;
            }
            let along_z = Vec2::new(ground.x, ground.y + step.y);
            if walkable(&terrain, along_z) {
                ground = along_z;
            }
            transform.translation.x = ground.x;
            transform.translation.z = ground.y;
            if heading.x.abs() > 0.05 {
                sprite.flip_x = heading.x < 0.0;
            }
            *facing = Facing::from_delta(heading.x.round() as i32, heading.y.round() as i32)
                .unwrap_or(*facing);
            stride.0 += delta_time * theme::STRIDE_RATE;
            let bounce = stride.0.sin().abs() * theme::STRIDE_BOUNCE;
            sprite.size = Vec2::new(1.0 - bounce * 0.5, 1.0 + bounce);
        } else {
            stride.0 = 0.0;
            sprite.size = Vec2::new(1.0, 1.0);
        }
        if let Some((column, row)) =
            crate::queries::terrain::column_of(&terrain, ground.x, ground.y)
        {
            let stand = crate::queries::terrain::stand(&terrain, column, row).y;
            let share = (theme::STEP_SMOOTHING * delta_time).clamp(0.0, 1.0);
            transform.translation.y += (stand - transform.translation.y) * share;
        }
        feet = transform.translation;
    });

    camp.feet = feet;
    look_at(&mut rig, feet);
}

pub fn find_the_prompt(
    mut camp: ResMut<Camp>,
    campaign: Res<Campaign>,
    fighters: Query<(&Called, &LocalTransform)>,
    rooms: Query<(&Room, &LocalTransform)>,
) {
    if campaign.phase != CampPhase::Roam {
        camp.prompt = Prompt::Nothing;
        return;
    }
    let feet = Vec2::new(camp.feet.x, camp.feet.z);
    let mut nearest: Option<(f32, Prompt)> = None;
    fighters.for_each(|entity, (name, transform)| {
        let ground = Vec2::new(transform.translation.x, transform.translation.z);
        let distance = (ground - feet).magnitude();
        if distance < theme::TALK_RADIUS
            && nearest.as_ref().is_none_or(|(best, _)| distance < *best)
        {
            nearest = Some((distance, Prompt::Talk(entity, name.0.to_string())));
        }
    });
    if nearest.is_none() {
        rooms.for_each(|_entity, (room, transform)| {
            let ground = Vec2::new(transform.translation.x, transform.translation.z);
            let distance = (ground - feet).magnitude();
            if distance < theme::ROOM_RADIUS
                && nearest.as_ref().is_none_or(|(best, _)| distance < *best)
            {
                nearest = Some((distance, Prompt::Enter(room.kind)));
            }
        });
    }
    camp.prompt = nearest.map_or(Prompt::Nothing, |(_, prompt)| prompt);
}

pub fn act_on_the_prompt(
    mut campaign: ResMut<Campaign>,
    mut camp: ResMut<Camp>,
    mut fighters: crate::queries::Fighters,
) {
    if campaign.phase != CampPhase::Roam {
        return;
    }
    if !camp.confirm {
        return;
    }
    let prompt = camp.prompt.clone();
    match prompt {
        Prompt::Nothing => {}
        Prompt::Talk(entity, _) => {
            let mut slot = 0;
            edit_fighter(&mut fighters, entity, |fighter| slot = fighter.slot);
            let companion = &COMPANIONS[slot.min(COMPANIONS.len() - 1)];
            let turn = (campaign.day as usize + slot) % companion.greetings.len();
            campaign.greeted = Some(entity);
            speak(
                &mut campaign,
                vec![(
                    companion.name.to_string(),
                    companion.greetings[turn].to_string(),
                )],
            );
        }
        Prompt::Enter(kind) => enter_the_room(&mut campaign, &mut fighters, kind),
    }
    camp.confirm = false;
}

pub fn speak_in_camp(mut campaign: ResMut<Campaign>, mut camp: ResMut<Camp>) {
    if campaign.phase != CampPhase::Speaking {
        return;
    }
    let (confirm, cancel) = (camp.confirm, camp.cancel);
    if !confirm && !cancel {
        return;
    }
    camp.confirm = false;
    camp.cancel = false;
    campaign.said += 1;
    if cancel || campaign.said >= campaign.speech.len() {
        finish_the_speech(&mut campaign);
    }
}

pub fn pass_the_day(mut campaign: ResMut<Campaign>, mut fighters: crate::queries::Fighters) {
    if campaign.actions > 0 || campaign.march || campaign.phase != CampPhase::Roam {
        return;
    }
    close_the_day(&mut campaign, &mut fighters);
}

pub fn camp_menu(
    handles: Res<UiHandles>,
    frame: Res<nightshade_ui::RetainedUiFrame>,
    mut campaign: ResMut<Campaign>,
    mut camp: ResMut<Camp>,
    mut fighters: crate::queries::Fighters,
) {
    if campaign.phase != CampPhase::Menu {
        return;
    }
    let Some(selected) = campaign.selected else {
        set_camp_phase(&mut campaign, CampPhase::Roam);
        return;
    };
    let squad = mustering(&mut fighters);
    let Some(index) = fighter_index(&squad, selected) else {
        set_camp_phase(&mut campaign, CampPhase::Roam);
        return;
    };
    let fighter = squad[index].1.clone();
    if camp.cancel {
        set_camp_phase(&mut campaign, CampPhase::Roam);
        return;
    }
    let limit = deploy_limit(&campaign);
    let ready = row_flags(&campaign, &squad, Some(index), limit);
    let Some(item) = choose_row(
        &mut campaign,
        &camp,
        &frame,
        &handles.camp_menu_buttons,
        &ready,
    ) else {
        return;
    };
    let Some(activity) = ACTIVITIES.get(item).copied() else {
        return;
    };
    match activity {
        Activity::Spar => {
            campaign.activity = Some(activity);
            campaign.partner_of = Some(selected);
            campaign.menu_index = 0;
            campaign.phase = CampPhase::Partner;
            story(&mut campaign, format!("Who spars with {}?", fighter.name));
            camp.confirm = false;
        }
        Activity::Advance => {
            campaign.menu_index = 0;
            campaign.phase = CampPhase::Advance;
            story(
                &mut campaign,
                format!("{} has training to spend.", fighter.name),
            );
            camp.confirm = false;
        }
        other => apply_alone(&mut campaign, &mut fighters, selected, fighter.slot, other),
    }
}

pub fn pick_the_partner(
    handles: Res<UiHandles>,
    frame: Res<nightshade_ui::RetainedUiFrame>,
    mut campaign: ResMut<Campaign>,
    camp: Res<Camp>,
    mut fighters: crate::queries::Fighters,
) {
    if campaign.phase != CampPhase::Partner {
        return;
    }
    let Some(host) = campaign.partner_of else {
        set_camp_phase(&mut campaign, CampPhase::Roam);
        return;
    };
    let squad = mustering(&mut fighters);
    if camp.cancel {
        campaign.selected = Some(host);
        set_camp_phase(&mut campaign, CampPhase::Menu);
        return;
    }
    let limit = deploy_limit(&campaign);
    let ready = row_flags(&campaign, &squad, None, limit);
    let Some(item) = choose_row(
        &mut campaign,
        &camp,
        &frame,
        &handles.camp_menu_buttons,
        &ready,
    ) else {
        return;
    };
    let guest = squad[item].0;
    let mut host_name = "";
    let mut guest_name = "";
    edit_fighter(&mut fighters, host, |fighter| {
        host_name = fighter.name;
        raise_bond(fighter, guest, theme::SPAR_BOND);
        fighter.busy = true;
        lift_morale(fighter, theme::MORALE_ACTIVITY);
        add_experience(fighter, theme::SPAR_EXPERIENCE);
    });
    edit_fighter(&mut fighters, guest, |fighter| {
        guest_name = fighter.name;
        raise_bond(fighter, host, theme::SPAR_BOND);
        fighter.busy = true;
        lift_morale(fighter, theme::MORALE_ACTIVITY);
        add_experience(fighter, theme::SPAR_EXPERIENCE);
    });
    spend(&mut campaign, Activity::Spar.actions(), 0);
    story(
        &mut campaign,
        format!("{host_name} and {guest_name} spar until dark."),
    );
    set_camp_phase(&mut campaign, CampPhase::Roam);
}

pub fn pick_the_upgrade(
    handles: Res<UiHandles>,
    frame: Res<nightshade_ui::RetainedUiFrame>,
    mut campaign: ResMut<Campaign>,
    camp: Res<Camp>,
    mut fighters: crate::queries::Fighters,
) {
    if campaign.phase != CampPhase::Advance {
        return;
    }
    let Some(selected) = campaign.selected else {
        set_camp_phase(&mut campaign, CampPhase::Roam);
        return;
    };
    let squad = mustering(&mut fighters);
    let Some(index) = fighter_index(&squad, selected) else {
        set_camp_phase(&mut campaign, CampPhase::Roam);
        return;
    };
    if camp.cancel {
        set_camp_phase(&mut campaign, CampPhase::Menu);
        return;
    }
    let options = upgrade_options(&squad[index].1);
    let ready = vec![true; options.len()];
    let Some(item) = choose_row(
        &mut campaign,
        &camp,
        &frame,
        &handles.camp_menu_buttons,
        &ready,
    ) else {
        return;
    };
    let Some(upgrade) = options.get(item).copied() else {
        return;
    };
    let mut line = String::new();
    edit_fighter(&mut fighters, selected, |fighter| {
        take_the_upgrade(fighter, upgrade);
        line = format!("{} {}", fighter.name, upgrade.line());
    });
    story(&mut campaign, line);
    set_camp_phase(&mut campaign, CampPhase::Menu);
}

pub fn muster_menu(
    handles: Res<UiHandles>,
    frame: Res<nightshade_ui::RetainedUiFrame>,
    mut campaign: ResMut<Campaign>,
    camp: Res<Camp>,
    mut fighters: crate::queries::Fighters,
) {
    if campaign.phase != CampPhase::Muster {
        return;
    }
    let squad = mustering(&mut fighters);
    if camp.cancel {
        set_camp_phase(&mut campaign, CampPhase::Roam);
        return;
    }
    let limit = deploy_limit(&campaign);
    let ready = row_flags(&campaign, &squad, None, limit);
    let Some(item) = choose_row(
        &mut campaign,
        &camp,
        &frame,
        &handles.camp_menu_buttons,
        &ready,
    ) else {
        return;
    };
    let entity = squad[item].0;
    let mut line = String::new();
    edit_fighter(&mut fighters, entity, |fighter| {
        fighter.deployed = !fighter.deployed;
        line = if fighter.deployed {
            format!("{} joins the muster.", fighter.name)
        } else {
            format!("{} stands down.", fighter.name)
        };
    });
    let limit = deploy_limit(&campaign);
    let squad = mustering(&mut fighters);
    if mustered(&squad) > limit {
        edit_fighter(&mut fighters, entity, |fighter| fighter.deployed = false);
        line = format!("The muster holds only {limit}.");
    }
    story(&mut campaign, line);
}

pub fn stir_the_speaker(
    time: Res<Time>,
    camp: Res<Camp>,
    campaign: Res<Campaign>,
    mut rig: ResMut<CameraRig>,
    people: Query<(&Called, &crate::data::Mood, &LocalTransform, &mut Sprite)>,
) {
    if campaign.phase == CampPhase::Roam {
        release_the_frame(&mut rig);
        people.for_each(|_entity, (_, _, _, sprite)| {
            sprite.size = Vec2::new(1.0, 1.0);
            sprite.anchor = Vec2::new(0.5, 0.0);
        });
        return;
    }
    let speaker = campaign
        .speech
        .get(campaign.said)
        .map(|(name, _)| name.clone())
        .unwrap_or_default();
    let talking = campaign.phase == CampPhase::Speaking;
    let clock = time.uptime_milliseconds as f32 / 1000.0;
    let mut looked_at: Option<Vec3> = None;
    people.for_each(|entity, (name, mood, transform, sprite)| {
        if Some(entity) == campaign.selected {
            looked_at = Some(transform.translation);
        }
        let stirred = talking && speaker.starts_with(name.0);
        if stirred {
            let (amplitude, rate, lean) = mood.bob();
            let wave = (clock * rate).sin();
            sprite.size = Vec2::new(1.0 - wave * amplitude * 0.5, 1.0 + wave * amplitude);
            sprite.anchor = Vec2::new(0.5 + wave * lean * 0.1, 0.0);
        } else {
            sprite.size = Vec2::new(1.0, 1.0);
            sprite.anchor = Vec2::new(0.5, 0.0);
        }
    });
    match looked_at {
        Some(there) => frame_the_talk(&mut rig, camp.feet, there),
        None => release_the_frame(&mut rig),
    }
}
