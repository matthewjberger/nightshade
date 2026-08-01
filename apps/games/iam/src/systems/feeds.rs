use crate::components::{ChannelFeed, FeedEntry};
use crate::data::script::{ChannelDefinition, Fact, Post, condition_holds};
use crate::resources::{AppResources, Ink, open_server};
use nightshade::prelude::Entity;
use nightshade::prelude::*;
use nightshade_platform::prelude::Time;
use nightshade_ui::commands::builder::UiBuild;
use nightshade_ui::prelude::*;

pub fn advance(
    mut resources: ResMut<AppResources>,
    time: Res<Time>,
    mut gathered: ResMut<crate::systems::desk::Gathered>,
    mut build: UiBuild,
    nodes: Query<&mut UiLayoutNode>,
    contents: Query<&mut UiNodeContent>,
) {
    let mut ink = Ink {
        build: build.context(),
        nodes,
        contents,
        gathered: &mut gathered,
    };
    let ink = &mut ink;
    let resources = &mut *resources;
    if !crate::data::machine::is_guest(
        crate::systems::desk::signed_in_account(&ink.gathered.desk),
        resources.desk.screen,
    ) {
        return;
    }
    let delta_time = time.delta_time;
    let channels: Vec<Entity> = ink
        .gathered
        .threads
        .channels
        .iter()
        .map(|state| state.entity)
        .collect();

    for entity in channels {
        tick_channel(resources, ink, entity, delta_time);
    }
}

enum Next {
    Reply,
    Held(usize),
    Scheduled,
    SetAside,
    Nothing,
}

fn tick_channel(
    resources: &mut AppResources,
    ink: &mut Ink<'_, '_, '_, '_, '_, '_>,
    entity: Entity,
    delta_time: f32,
) {
    let Some(channel) = crate::systems::conversation::channel_at(&ink.gathered.threads, entity)
        .map(|state| state.channel.clone())
    else {
        return;
    };
    let listed = crate::systems::conversation::server_at(&ink.gathered.threads, channel.server)
        .is_some_and(|state| state.listed);
    if !listed {
        return;
    }

    let script = resources.script;
    let definition = &script.servers[channel.server_definition].channels[channel.definition];
    let Some(feed) = crate::systems::conversation::channel_at(&ink.gathered.threads, entity)
        .map(|state| state.feed.clone())
    else {
        return;
    };
    let night = resources.session.night;
    let facts = &resources.session.facts;

    let choice = choose(&feed, definition, night, facts);
    if let Next::SetAside = choice {
        if let Some(feed) =
            crate::systems::conversation::channel_mut(&mut ink.gathered.threads, entity)
                .map(|state| &mut state.feed)
        {
            let index = feed.delivered;
            feed.held.push(index);
            feed.delivered += 1;
        }
        return;
    }
    let post: &Post = match choice {
        Next::Reply => feed.queue[0],
        Next::Held(index) => &definition.posts[index],
        Next::Scheduled => &definition.posts[feed.delivered],
        _ => return,
    };

    let timer = feed.timer - delta_time;
    if timer > 0.0 {
        if let Some(feed) =
            crate::systems::conversation::channel_mut(&mut ink.gathered.threads, entity)
                .map(|state| &mut state.feed)
        {
            feed.timer = timer;
        }
        return;
    }

    let landed = FeedEntry {
        author: post.author,
        text: post.text,
        stamp: crate::systems::interface::day_stamp(resources),
        mine: false,
    };
    let from_queue = matches!(choice, Next::Reply);
    let finished_replies = from_queue && feed.queue.len() == 1;
    let effects = feed.queued_effects;

    if let Some(feed) = crate::systems::conversation::channel_mut(&mut ink.gathered.threads, entity)
        .map(|state| &mut state.feed)
    {
        feed.log.push(landed);
        match choice {
            Next::Reply => {
                feed.queue.remove(0);
            }
            Next::Held(index) => feed.held.retain(|held| *held != index),
            _ => feed.delivered += 1,
        }
        if finished_replies {
            feed.queued_effects = &[];
        }
    }

    if let Some(feed) = crate::systems::conversation::channel_at(&ink.gathered.threads, entity)
        .map(|state| state.feed.clone())
    {
        let delay = match choose(&feed, definition, night, facts) {
            Next::Reply => feed.queue[0].delay_seconds,
            Next::Held(index) => delay_for(&definition.posts[index], night),
            Next::Scheduled => delay_for(&definition.posts[feed.delivered], night),
            _ => 0.0,
        };
        if let Some(feed) =
            crate::systems::conversation::channel_mut(&mut ink.gathered.threads, entity)
                .map(|state| &mut state.feed)
        {
            feed.timer = delay;
        }
    }

    if finished_replies {
        crate::systems::conversation::apply_effects(resources, ink, effects);
    }

    let showing = open_server(&resources.session) == Some(channel.server)
        && crate::systems::conversation::server_at(&ink.gathered.threads, channel.server)
            .and_then(|state| state.open_channel)
            == Some(entity);
    if !showing
        && let Some(state) =
            crate::systems::conversation::channel_mut(&mut ink.gathered.threads, entity)
    {
        state.unread += 1;
    }
    if from_queue {
        let where_it_is = format!("{}  Â·  {}", post.author, definition.name);
        crate::systems::desk::raise_toast(
            resources,
            ink,
            crate::resources::ToastTarget::Room {
                server: channel.server,
                channel: entity,
            },
            &where_it_is,
            post.text,
        );
    }
    resources.redraw.sidebar = true;
    resources.redraw.header = true;
}

fn choose(
    feed: &ChannelFeed,
    definition: &ChannelDefinition,
    night: usize,
    facts: &[Fact],
) -> Next {
    if !feed.queue.is_empty() {
        return Next::Reply;
    }
    for index in &feed.held {
        if condition_holds(&definition.posts[*index].condition, facts) {
            return Next::Held(*index);
        }
    }
    let Some(post) = definition.posts.get(feed.delivered) else {
        return Next::Nothing;
    };
    if post.night > night {
        return Next::Nothing;
    }
    if !condition_holds(&post.condition, facts) {
        if post.night < night {
            return Next::SetAside;
        }
        return Next::Nothing;
    }
    Next::Scheduled
}

fn delay_for(post: &Post, night: usize) -> f32 {
    if post.night < night {
        0.0
    } else {
        post.delay_seconds
    }
}

pub fn server_unread(ink: &Ink<'_, '_, '_, '_, '_, '_>, server: Entity) -> u32 {
    ink.gathered
        .threads
        .channels
        .iter()
        .filter(|state| state.channel.server == server)
        .map(|state| state.unread)
        .sum()
}
