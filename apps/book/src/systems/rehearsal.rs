use nightshade::prelude::*;
use nightshade_graphics::prelude::CommandQueues;
use nightshade_ui_test::UiTestDriver;

use crate::resources::Rehearsal;

const OPEN: u64 = 30;
const REST_ON_DRACULA: u64 = 36;
const PICTURE_DRACULA_PREVIEW: u64 = 70;
const TAKE_DRACULA: u64 = 72;
const PICTURE_DRACULA_TAKEN: u64 = 120;
const REST_ON_NORD: u64 = 122;
const PICTURE_NORD_PREVIEW: u64 = 160;
const REST_ON_LARGE: u64 = 162;
const PICTURE_LARGE_PREVIEW: u64 = 195;
const TAKE_LARGE: u64 = 197;
const PICTURE_LARGE_TAKEN: u64 = 255;
const PRESS_THE_PAGE: u64 = 257;
const PICTURE_CLOSED: u64 = 290;
const OPEN_THE_LIST: u64 = 292;
const PICTURE_LIST_OPEN: u64 = 325;
const PRESS_THE_PAGE_AGAIN: u64 = 327;
const OPEN_THE_SOURCE: u64 = 335;
const PICTURE_SOURCE: u64 = 380;
const LAST: u64 = 390;

const SETTLE: u64 = 4;

const DRACULA: &str = "theme-2";
const NORD: &str = "theme-3";
const LARGE: &str = "size-2";
const LIST: &str = "figure-1-control-0";
const SOURCE: &str = "figure-1-source";

pub fn script() -> UiTestDriver {
    let mut driver = UiTestDriver::new("book appearance");
    driver.set_exit_on_complete(false);
    driver.at_frame(OPEN).click_entity("appearance");
    driver.wait(SETTLE).assert_visible("appearance-panel");
    driver.at_frame(REST_ON_DRACULA).move_mouse_to(DRACULA);
    driver.at_frame(TAKE_DRACULA).click_entity(DRACULA);
    driver.wait(SETTLE).assert_visible("appearance-panel");
    driver.at_frame(REST_ON_NORD).move_mouse_to(NORD);
    driver.at_frame(REST_ON_LARGE).move_mouse_to(LARGE);
    driver.at_frame(TAKE_LARGE).click_entity(LARGE);
    driver.wait(SETTLE).assert_visible("appearance-panel");
    driver.at_frame(PRESS_THE_PAGE).click_entity("content");
    driver.wait(SETTLE).assert_not_visible("appearance-panel");
    driver.at_frame(OPEN_THE_LIST).click_entity(LIST);
    driver
        .at_frame(PRESS_THE_PAGE_AGAIN)
        .click_entity("content");
    driver.at_frame(OPEN_THE_SOURCE).click_entity(SOURCE);
    driver.wait(SETTLE).assert_visible("source-panel");
    driver.start();
    driver
}

fn picture_at(frame: u64) -> Option<&'static str> {
    match frame {
        PICTURE_DRACULA_PREVIEW => Some("a_dracula_preview.png"),
        PICTURE_DRACULA_TAKEN => Some("b_dracula_taken.png"),
        PICTURE_NORD_PREVIEW => Some("c_nord_preview.png"),
        PICTURE_LARGE_PREVIEW => Some("d_large_preview.png"),
        PICTURE_LARGE_TAKEN => Some("e_large_taken.png"),
        PICTURE_CLOSED => Some("f_closed.png"),
        PICTURE_LIST_OPEN => Some("g_list_open.png"),
        PICTURE_SOURCE => Some("h_source.png"),
        _ => None,
    }
}

pub fn step(
    mut rehearsal: ResMut<Rehearsal>,
    mut queues: ResMut<CommandQueues>,
    mut stage: nightshade_ui_test::TestStage,
    storage: &mut nightshade::ecs::Storage,
) {
    let Some(mut driver) = rehearsal.driver.take() else {
        return;
    };
    driver.step(storage, &mut stage);
    rehearsal.frame += 1;
    let frame = rehearsal.frame;
    let directory = rehearsal.directory.clone();
    if let Some(name) = picture_at(frame) {
        nightshade_graphics::commands::queues::capture_screenshot_to_path(
            &mut queues,
            directory.join(name),
        );
    }
    if frame >= LAST {
        stage.window.should_exit = true;
    }
    rehearsal.driver = Some(driver);
}
