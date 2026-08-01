use crate::data::ask::Ask;
use crate::data::battle::Battle;
use crate::queries::battle::zone_len;
use crate::resources::{Cursor, Zone};
use nightshade_input::Actions;
use nightshade_platform::prelude::*;

pub fn move_cursor(input: &Input, actions: &Actions<Ask>, battle: &Battle, cursor: &mut Cursor) {
    if input.mouse.position_delta.magnitude_squared() > 0.5 {
        cursor.active = false;
    }
    let stick_x = actions.axis2(Ask::CursorSlide).x;
    let mut step: i32 = 0;
    if actions.just_pressed(Ask::CursorLeft) {
        step -= 1;
    }
    if actions.just_pressed(Ask::CursorRight) {
        step += 1;
    }
    let mut zone_step: i32 = 0;
    if actions.just_pressed(Ask::ZoneUp) {
        zone_step += 1;
    }
    if actions.just_pressed(Ask::ZoneDown) {
        zone_step -= 1;
    }
    let stick_now = if stick_x > 0.6 {
        1
    } else if stick_x < -0.6 {
        -1
    } else {
        0
    };
    if stick_now != 0 && stick_now != cursor.stick_latched {
        step += stick_now;
    }
    cursor.stick_latched = stick_now;

    if step == 0 && zone_step == 0 {
        return;
    }
    cursor.active = true;
    if zone_step != 0 {
        let zones = [Zone::Hand, Zone::Board, Zone::Enemy];
        let current = zones
            .iter()
            .position(|zone| *zone == cursor.zone)
            .unwrap_or(0) as i32;
        let next = (current + zone_step).clamp(0, 2) as usize;
        cursor.zone = zones[next];
    }
    let len = zone_len(battle, cursor.zone);
    if len == 0 {
        cursor.index = 0;
        return;
    }
    let index = (cursor.index as i32 + step).rem_euclid(len as i32);
    cursor.index = index as usize;
}
