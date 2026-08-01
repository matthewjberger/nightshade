use nightshade::prelude::*;
use nightshade_ui::commands::builder::UiBuild;
use nightshade_ui::prelude::*;

use crate::resources::{BookState, Library, PagePhase, Search, SearchEntry, Shell};

pub fn collect(mut search: ResMut<Search>, library: Res<Library>) {
    if !search.entries.is_empty() || library.chapters.is_empty() {
        return;
    }

    for (index, chapter) in library.chapters.iter().enumerate() {
        search.entries.push(SearchEntry {
            label: format!("{}  {}", chapter.number, chapter.title),
            chapter: index,
            anchor: None,
        });
        for heading in &chapter.outline {
            search.entries.push(SearchEntry {
                label: format!("{}  \u{203a}  {}", chapter.title, heading.title),
                chapter: index,
                anchor: Some(heading.anchor.clone()),
            });
        }
    }
}

pub fn drive(
    mut search: ResMut<Search>,
    mut state: ResMut<BookState>,
    shell: Res<Shell>,
    mut build: UiBuild,
    storage: &mut nightshade::ecs::Storage,
) {
    let mut picked = None;
    for event in ui_events(build.frame()) {
        if let UiEvent::CommandPaletteExecuted { command_index, .. } = event {
            picked = Some(*command_index);
        }
    }

    if let Some(index) = picked {
        search.open = false;
        if let Some(entry) = search.entries.get(index).cloned() {
            if entry.chapter != state.current && state.phase == PagePhase::Idle {
                state.requested = Some(entry.chapter);
            }
            state.jump_to = entry.anchor;
        }
        return;
    }

    if !search.open {
        return;
    }
    search.open = false;

    let mut context = build.context();
    if !search.filled {
        for entry in &search.entries {
            ui_command_palette_register(&mut context, shell.palette, &entry.label, "", "");
        }
        search.filled = true;
    }
    ui_show_command_palette(&mut context, storage, shell.palette);
}
