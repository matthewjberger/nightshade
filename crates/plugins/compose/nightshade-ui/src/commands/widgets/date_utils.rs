use crate::data::state::UiStateTrait;
use nalgebra_glm;

pub(crate) fn format_month_year(year: i32, month: u32) -> String {
    let month_name = match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "???",
    };
    format!("{month_name} {year}")
}

pub(crate) fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

pub(crate) fn day_of_week(year: i32, month: u32, day: u32) -> u32 {
    let (adjusted_year, adjusted_month) = if month <= 2 {
        (year - 1, month + 12)
    } else {
        (year, month)
    };
    let day_of_month = day as i32;
    let month_index = adjusted_month as i32;
    let year_of_century = adjusted_year.rem_euclid(100);
    let century = adjusted_year.div_euclid(100);
    let weekday = (day_of_month
        + (13 * (month_index + 1)) / 5
        + year_of_century
        + year_of_century / 4
        + century / 4
        - 2 * century)
        .rem_euclid(7);
    ((weekday + 6) % 7) as u32
}

pub(crate) fn populate_calendar_grid(
    text_state: &mut nightshade_text::contract::resources::TextState,
    commands: &mut nightshade::ecs::system_param::Commands<'_>,
    year: i32,
    month: u32,
    selected_day: u32,
    day_text_slots: &[usize],
    day_entities: &[nightshade::ecs::Entity],
    accent_color: nalgebra_glm::Vec4,
) {
    let dim_color = nalgebra_glm::Vec4::new(0.5, 0.5, 0.5, 0.3);
    let clear_color = nalgebra_glm::Vec4::new(0.0, 0.0, 0.0, 0.0);

    let first_day_of_week = day_of_week(year, month, 1);
    let days = days_in_month(year, month);

    let (previous_year, previous_month) = if month == 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
    };
    let previous_month_days = days_in_month(previous_year, previous_month);

    for cell_index in 0..42 {
        let (day_number, is_current_month) = if (cell_index as u32) < first_day_of_week {
            let day = previous_month_days - first_day_of_week + cell_index as u32 + 1;
            (day, false)
        } else {
            let day = cell_index as u32 - first_day_of_week + 1;
            if day <= days {
                (day, true)
            } else {
                let day = day - days;
                (day, false)
            }
        };

        text_state
            .cache
            .set_text(day_text_slots[cell_index], day_number.to_string());

        let is_selected = is_current_month && day_number == selected_day;
        let base_color = if is_selected {
            accent_color
        } else if is_current_month {
            clear_color
        } else {
            dim_color
        };
        let day_entity = day_entities[cell_index];
        commands.queue(move |storage, _| {
            if let Some(color) = storage.get_mut::<crate::components::UiNodeColor>(day_entity) {
                color.colors[crate::data::state::UiBase::INDEX] = Some(base_color);
            }
        });
    }
}
