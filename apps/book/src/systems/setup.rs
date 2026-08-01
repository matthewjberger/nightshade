use nightshade::prelude::*;
use nightshade_render::data::config::{DebugDraw, RenderSettings};

use crate::commands::book::load_library;
use crate::commands::theme::install_book_themes;
use crate::queries::book::requested_page;
use crate::resources::{Appearance, BookArguments, BookState, Library};

pub fn initialize(
    mut library: ResMut<Library>,
    mut book: ResMut<BookState>,
    mut settings: ResMut<RenderSettings>,
    mut debug_draw: ResMut<DebugDraw>,
    mut theme: ResMut<nightshade_ui::resources::ThemeState>,
    mut appearance: ResMut<Appearance>,
    arguments: Res<BookArguments>,
) {
    install_book_themes(&mut theme);
    load_library(&mut library);
    if let Some(page) = requested_page(&library, arguments.page.as_deref()) {
        book.requested = Some(page);
    }
    book.scroll_to = arguments.scroll;
    appearance.asked = arguments.appearance;

    debug_draw.show_grid = false;
    settings.render_world_to_swapchain = false;
}
