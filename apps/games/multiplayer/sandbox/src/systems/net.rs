use crate::commands::players::{spawn_driven, spawn_tracked};
use crate::commands::things::spawn_thing;
use crate::components::{
    At, Avatar, Crouch, Facing, Fade, Grounded, JumpHold, Look, Peer, Replica, Steering, Track,
    Velocity, Wants, WasAt,
};
use crate::data::wire::{Body, PeerId, Request, Seat, Thing, Update};
use crate::data::{CHARACTERS, KIND_MARK};
use crate::queries::aim::{bodies_at, tag_target};
use crate::queries::step::{Motion, Wish, step_character};
use crate::queries::{character_of, label, spawn_of};
use crate::resources::{Incoming, Outgoing, Role, Sent, Session, Shot, Wire};
use crate::theme;
use nalgebra_glm::Vec3;
use nightshade::prelude::*;
use nightshade_physics::prelude::PhysicsWorld;
use nightshade_platform::prelude::Time;
use nightshade_scene::prelude::LocalTransform;

pub fn pump(
    mut wire: ResMut<Wire>,
    mut session: ResMut<Session>,
    mut commands: Commands,
    mut steering: Query<(&mut Steering, &mut Wants)>,
    mut bodies: Query<(&Look, &Avatar)>,
    mut marks: Query<(&Replica, &mut LocalTransform)>,
) {
    let mut arrived: Vec<(PeerId, String)> = Vec::new();
    let mut departed: Vec<PeerId> = Vec::new();
    let mut chats: Vec<(PeerId, String)> = Vec::new();
    let mut snapshots: Vec<Vec<Body>> = Vec::new();
    let mut welcomes: Vec<(PeerId, u8, Vec<Seat>, Vec<Thing>)> = Vec::new();
    let mut seated: Vec<Seat> = Vec::new();
    let mut tagged: Vec<(PeerId, PeerId)> = Vec::new();
    let mut appeared: Vec<Thing> = Vec::new();
    let mut vanished: Vec<u32> = Vec::new();
    let mut moved: Vec<Thing> = Vec::new();

    while let Ok(event) = wire.from_net.try_recv() {
        match event {
            Incoming::Note(note) => session.note = note,
            Incoming::Departed(peer) => departed.push(peer),
            Incoming::FromGuest(peer, Request::Hello { name }) => arrived.push((peer, name)),
            Incoming::FromGuest(peer, Request::Chat { text }) => chats.push((peer, text)),
            Incoming::FromGuest(
                peer,
                Request::Tag {
                    tick,
                    origin,
                    direction,
                },
            ) => session.shots.push(Shot {
                peer,
                tick,
                origin: Vec3::new(origin[0], origin[1], origin[2]),
                direction: Vec3::new(direction[0], direction[1], direction[2]),
            }),
            Incoming::FromGuest(
                peer,
                Request::Input {
                    sequence,
                    walk,
                    wants,
                },
            ) => {
                let direction = Vec3::new(walk[0], 0.0, walk[1]);
                session.latest.insert(peer, (sequence, direction, wants));
            }
            Incoming::FromHost(Update::Welcome {
                you,
                character,
                roster,
                things,
            }) => welcomes.push((you, character, roster, things)),
            Incoming::FromHost(Update::Joined { seat }) => seated.push(seat),
            Incoming::FromHost(Update::Left { peer }) => departed.push(peer),
            Incoming::FromHost(Update::Chat { peer, text }) => chats.push((peer, text)),
            Incoming::FromHost(Update::Tagged { by, peer }) => tagged.push((by, peer)),
            Incoming::FromHost(Update::Snapshot {
                tick,
                bodies,
                things,
            }) => {
                session.tick = tick;
                snapshots.push(bodies);
                moved.extend(things);
            }
            Incoming::FromHost(Update::Spawned { thing }) => appeared.push(thing),
            Incoming::FromHost(Update::Despawned { id }) => vanished.push(id),
        }
    }

    for (peer, name) in arrived {
        let fresh = !session.bodies.contains_key(&peer);
        if fresh {
            let seat = session.seats.len();
            let character = character_of(seat);
            session.seats.push(Seat {
                peer,
                name: name.clone(),
                character,
            });
            let body = spawn_driven(&mut commands, peer, character, spawn_of(seat));
            session.bodies.insert(peer, body);
            let _ = wire.to_net.send(Outgoing::Everyone(Update::Joined {
                seat: Seat {
                    peer,
                    name: name.clone(),
                    character,
                },
            }));
            session.chat.push(format!("{name} joined"));
        }
        let Some(seat) = session.seats.iter().find(|seat| seat.peer == peer) else {
            continue;
        };
        let character = seat.character;
        let roster = session.seats.clone();
        let things = session.replicas.clone();
        let _ = wire.to_net.send(Outgoing::One(
            peer,
            Update::Welcome {
                you: peer,
                character,
                roster,
                things,
            },
        ));
    }

    for (you, character, roster, things) in welcomes {
        session.seats = roster;
        appeared.extend(things);
        session.note = "playing".to_string();
        for seat in session.seats.clone() {
            if seat.peer == you {
                continue;
            }
            open_seat(&mut session, &mut commands, seat);
        }
        if let Some(body) = session.local {
            let wanted = crate::queries::model(CHARACTERS[character as usize % CHARACTERS.len()]);
            let mut worn = None;
            let mut hung = None;
            bodies.for_entity(body, |(look, avatar)| {
                worn = Some(look.0);
                hung = avatar.0;
            });
            if worn != Some(wanted) {
                if let Some(old) = hung {
                    commands.despawn_recursive(old);
                }
                commands.queue(move |storage, _resources| {
                    storage.set(body, Look(wanted));
                    storage.set(body, Avatar(None));
                });
            }
        }
    }

    for seat in seated {
        if seat.peer == wire.peer {
            continue;
        }
        session.chat.push(format!("{} joined", seat.name));
        if !session.seats.iter().any(|known| known.peer == seat.peer) {
            session.seats.push(seat.clone());
        }
        open_seat(&mut session, &mut commands, seat);
    }

    for peer in departed {
        if let Some(body) = session.bodies.remove(&peer) {
            commands.despawn_recursive(body);
        }
        if let Some(index) = session.seats.iter().position(|seat| seat.peer == peer) {
            let name = session.seats.remove(index).name;
            session.chat.push(format!("{name} left"));
        }
        session.latest.remove(&peer);
        if wire.role == Role::Host {
            let _ = wire.to_net.send(Outgoing::Everyone(Update::Left { peer }));
        }
    }

    for (peer, text) in chats {
        if wire.role == Role::Guest && peer == wire.peer {
            continue;
        }
        let who = name_of(&session, peer);
        session.chat.push(format!("{who}: {text}"));
        if wire.role == Role::Host {
            let _ = wire
                .to_net
                .send(Outgoing::Everyone(Update::Chat { peer, text }));
        }
    }

    if wire.role == Role::Host && !session.shots.is_empty() {
        let shots = std::mem::take(&mut session.shots);
        let hits: Vec<(PeerId, PeerId, Vec3)> = shots
            .iter()
            .filter_map(|shot| {
                let bodies = bodies_at(&session.history, shot.tick)?;
                let (hit, point) = tag_target(shot.peer, shot.origin, shot.direction, bodies)?;
                Some((shot.peer, hit, point))
            })
            .collect();
        for (by, peer, point) in hits {
            let _ = wire
                .to_net
                .send(Outgoing::Everyone(Update::Tagged { by, peer }));
            tagged.push((by, peer));
            let id = session.next_thing;
            session.next_thing = session.next_thing.wrapping_add(1);
            let thing = Thing {
                id,
                kind: KIND_MARK,
                position: [point.x, point.y, point.z],
                facing: 0.0,
            };
            let Some(entity) = spawn_thing(&mut commands, thing) else {
                continue;
            };
            commands.queue(move |storage, _resources| {
                storage.set(entity, Fade(theme::MARK_SECONDS));
            });
            session.things.insert(id, entity);
            session.replicas.push(thing);
            let _ = wire
                .to_net
                .send(Outgoing::Everyone(Update::Spawned { thing }));
        }
    }

    for thing in appeared {
        if session.things.contains_key(&thing.id) {
            continue;
        }
        if let Some(entity) = spawn_thing(&mut commands, thing) {
            session.things.insert(thing.id, entity);
        }
    }

    for id in vanished {
        if let Some(entity) = session.things.remove(&id) {
            commands.despawn_recursive(entity);
        }
    }

    for thing in moved {
        let Some(&entity) = session.things.get(&thing.id) else {
            continue;
        };
        marks.for_entity(entity, |(_replica, transform)| {
            transform.translation =
                Vec3::new(thing.position[0], thing.position[1], thing.position[2]);
            transform.rotation = nalgebra_glm::quat_angle_axis(thing.facing, &Vec3::y());
        });
    }

    for (by, peer) in tagged {
        let shooter = name_of(&session, by);
        let target = name_of(&session, peer);
        session.chat.push(format!("{shooter} tagged {target}"));
    }

    let limit = session.chat.len().saturating_sub(theme::CHAT_LIMIT);
    if limit > 0 {
        session.chat.drain(..limit);
    }

    if wire.role == Role::Host {
        let orders: Vec<(PeerId, Vec3, u8)> = session
            .latest
            .iter()
            .map(|(peer, (_, walk, wants))| (*peer, *walk, *wants))
            .collect();
        for (peer, walk, wants) in orders {
            let Some(&body) = session.bodies.get(&peer) else {
                continue;
            };
            steering.for_entity(body, |(steer, want)| {
                steer.0 = walk;
                want.0 = wants;
            });
        }
    }

    let mut fresh: Vec<Body> = Vec::new();
    for snapshot in snapshots {
        for body in snapshot {
            match fresh.iter_mut().find(|known| known.peer == body.peer) {
                Some(known) => *known = body,
                None => fresh.push(body),
            }
        }
    }
    if !fresh.is_empty() {
        session.spawned = fresh;
    }
}

fn name_of(session: &Session, peer: PeerId) -> String {
    session
        .seats
        .iter()
        .find(|seat| seat.peer == peer)
        .map(|seat| seat.name.clone())
        .unwrap_or_else(|| label(peer))
}

fn open_seat(session: &mut Session, commands: &mut Commands, seat: Seat) {
    if session.bodies.contains_key(&seat.peer) {
        return;
    }
    let index = session
        .seats
        .iter()
        .position(|known| known.peer == seat.peer)
        .unwrap_or(0);
    let body = spawn_tracked(commands, seat.peer, seat.character, spawn_of(index));
    session.bodies.insert(seat.peer, body);
}

pub fn serve(
    wire: Res<Wire>,
    mut session: ResMut<Session>,
    bodies: Query<(&Peer, &At, &Velocity, &Facing, &Crouch, &Grounded)>,
) {
    if wire.role != Role::Host || !session.stepping {
        return;
    }
    session.tick = session.tick.wrapping_add(1);

    let acknowledged = session.latest.clone();
    let mut frame = Vec::new();
    bodies.for_each(|_entity, (peer, at, velocity, facing, crouch, grounded)| {
        frame.push(Body {
            peer: peer.0,
            position: [at.0.x, at.0.y, at.0.z],
            velocity: [velocity.0.x, velocity.0.y, velocity.0.z],
            facing: facing.0,
            crouch: crouch.0,
            grounded: grounded.0,
            acknowledged: acknowledged
                .get(&peer.0)
                .map(|(seq, _, _)| *seq)
                .unwrap_or(0),
        });
    });
    if frame.is_empty() {
        return;
    }
    let tick = session.tick;
    let things = session.replicas.clone();
    session.history.push_back((tick, frame.clone()));
    while session.history.len() > theme::HISTORY_TICKS {
        session.history.pop_front();
    }

    let audience: Vec<PeerId> = session
        .seats
        .iter()
        .map(|seat| seat.peer)
        .filter(|peer| *peer != wire.peer)
        .collect();
    session.mailed.retain(|peer, _| audience.contains(peer));
    for peer in audience {
        let Some(around) = frame
            .iter()
            .find(|body| body.peer == peer)
            .map(|body| Vec3::new(body.position[0], body.position[1], body.position[2]))
        else {
            continue;
        };
        let mailbox = session.mailed.entry(peer).or_default();
        let mut fresh_bodies = Vec::new();
        for body in &frame {
            if body.peer != peer && !near(around, body.position) {
                continue;
            }
            if mailbox.bodies.get(&body.peer) == Some(body) {
                continue;
            }
            mailbox.bodies.insert(body.peer, *body);
            fresh_bodies.push(*body);
        }
        let mut fresh_things = Vec::new();
        for thing in &things {
            if !near(around, thing.position) {
                continue;
            }
            if mailbox.things.get(&thing.id) == Some(thing) {
                continue;
            }
            mailbox.things.insert(thing.id, *thing);
            fresh_things.push(*thing);
        }
        let _ = wire.to_net.send(Outgoing::One(
            peer,
            Update::Snapshot {
                tick,
                bodies: fresh_bodies,
                things: fresh_things,
            },
        ));
    }
}

fn near(around: Vec3, position: [f32; 3]) -> bool {
    let offset = Vec3::new(position[0], position[1], position[2]) - around;
    offset.norm() <= theme::INTEREST_RADIUS
}

pub fn expire(
    time: Res<Time>,
    wire: Res<Wire>,
    mut session: ResMut<Session>,
    mut commands: Commands,
    marks: Query<(&Replica, &mut Fade)>,
) {
    if wire.role != Role::Host {
        return;
    }
    let delta = time.delta_time;
    let mut gone: Vec<u32> = Vec::new();
    marks.for_each(|_entity, (replica, fade)| {
        fade.0 -= delta;
        if fade.0 <= 0.0 {
            gone.push(replica.0);
        }
    });
    for id in gone {
        if let Some(entity) = session.things.remove(&id) {
            commands.despawn_recursive(entity);
        }
        session.replicas.retain(|thing| thing.id != id);
        let _ = wire
            .to_net
            .send(Outgoing::Everyone(Update::Despawned { id }));
    }
}

pub fn report(
    wire: Res<Wire>,
    mut session: ResMut<Session>,
    mut steering: Query<(&Steering, &Wants)>,
) {
    if wire.role != Role::Guest || !session.stepping {
        return;
    }

    if session.seats.is_empty() {
        let name = session.name.clone();
        let _ = wire.to_net.send(Outgoing::Host(Request::Hello { name }));
        return;
    }

    let Some(body) = session.local else {
        return;
    };
    let mut wish = (Vec3::zeros(), 0_u8);
    steering.for_entity(body, |(steer, want)| wish = (steer.0, want.0));

    session.sequence = session.sequence.wrapping_add(1);
    let sequence = session.sequence;
    let _ = wire.to_net.send(Outgoing::Host(Request::Input {
        sequence,
        walk: [wish.0.x, wish.0.z],
        wants: wish.1,
    }));
    session.sent.push(Sent {
        sequence,
        walk: wish.0,
        wants: wish.1,
    });
    let overflow = session.sent.len().saturating_sub(theme::SAVED_INPUTS);
    if overflow > 0 {
        session.sent.drain(..overflow);
    }
}

pub fn aim(wire: Res<Wire>, mut session: ResMut<Session>) {
    if !session.fired {
        return;
    }
    session.fired = false;
    if session.local.is_none() || session.seats.is_empty() {
        return;
    }
    let origin = session.aim_origin;
    let direction = session.aim_direction;
    match wire.role {
        Role::Host => {
            let tick = session.tick;
            let peer = wire.peer;
            session.shots.push(Shot {
                peer,
                tick,
                origin,
                direction,
            });
        }
        Role::Guest => {
            let delay = (theme::INTERPOLATION_DELAY * theme::TICK_HZ) as u32;
            let tick = session.tick.saturating_sub(delay);
            let _ = wire.to_net.send(Outgoing::Host(Request::Tag {
                tick,
                origin: [origin.x, origin.y, origin.z],
                direction: [direction.x, direction.y, direction.z],
            }));
        }
    }
}

pub fn reconcile(
    wire: Res<Wire>,
    physics: Res<PhysicsWorld>,
    mut session: ResMut<Session>,
    mut bodies: Query<(
        &mut At,
        &mut WasAt,
        &mut Velocity,
        &mut Facing,
        &mut Crouch,
        &mut Grounded,
        &mut JumpHold,
    )>,
) {
    if wire.role != Role::Guest || session.spawned.is_empty() {
        return;
    }
    let Some(body) = session.local else {
        return;
    };
    let Some(mine) = session
        .spawned
        .iter()
        .find(|state| state.peer == wire.peer)
        .copied()
    else {
        return;
    };

    session.sent.retain(|sent| {
        sent.sequence.wrapping_sub(mine.acknowledged) < u32::MAX / 2
            && sent.sequence != mine.acknowledged
    });

    let replay: Vec<Sent> = if session.predict {
        session.sent.clone()
    } else {
        Vec::new()
    };

    bodies.for_entity(
        body,
        |(at, was_at, velocity, facing, crouch, grounded, hold)| {
            let mut motion = Motion {
                position: Vec3::new(mine.position[0], mine.position[1], mine.position[2]),
                velocity: Vec3::new(mine.velocity[0], mine.velocity[1], mine.velocity[2]),
                facing: mine.facing,
                crouch: mine.crouch,
                grounded: mine.grounded,
                jump_hold: hold.0,
            };
            for sent in &replay {
                motion = step_character(
                    motion,
                    Wish {
                        walk: sent.walk,
                        wants: sent.wants,
                    },
                    theme::TICK_SECONDS,
                    &physics,
                );
            }
            let error = motion.position - at.0;
            let correction = if error.norm() > theme::CORRECTION_SNAP {
                error
            } else {
                error * theme::CORRECTION_BLEND
            };
            at.0 += correction;
            was_at.0 += correction;
            velocity.0 = motion.velocity;
            facing.0 = motion.facing;
            crouch.0 = motion.crouch;
            grounded.0 = motion.grounded;
            hold.0 = motion.jump_hold;
        },
    );
}

pub fn file(mut session: ResMut<Session>, tracks: Query<(&Peer, &mut Track)>) {
    if session.spawned.is_empty() {
        return;
    }
    let arrived = std::mem::take(&mut session.spawned);
    let clock = session.clock;
    tracks.for_each(|_entity, (peer, track)| {
        let Some(state) = arrived.iter().find(|body| body.peer == peer.0) else {
            return;
        };
        track.0.push_back((clock, *state));
        while track.0.len() > theme::TRACK_DEPTH {
            track.0.pop_front();
        }
    });
}

pub fn follow(
    session: Res<Session>,
    bodies: Query<(
        &Track,
        &mut LocalTransform,
        &mut Velocity,
        &mut Facing,
        &mut Crouch,
        &mut Grounded,
    )>,
) {
    let render_time = session.clock - theme::INTERPOLATION_DELAY;
    let smooth = session.interpolate;
    bodies.for_each(
        |_entity, (track, transform, velocity, facing, crouch, grounded)| {
            let samples = &track.0;
            let Some((_, newest)) = samples.back() else {
                return;
            };
            let mut position =
                Vec3::new(newest.position[0], newest.position[1], newest.position[2]);
            let mut turn = newest.facing;
            if smooth {
                let mut pair = None;
                for (before, after) in samples.iter().zip(samples.iter().skip(1)) {
                    if before.0 <= render_time && render_time <= after.0 {
                        pair = Some((*before, *after));
                    }
                }
                if let Some((before, after)) = pair {
                    let span = (after.0 - before.0).max(1e-4);
                    let blend = ((render_time - before.0) / span).clamp(0.0, 1.0);
                    let start = Vec3::new(
                        before.1.position[0],
                        before.1.position[1],
                        before.1.position[2],
                    );
                    let end = Vec3::new(
                        after.1.position[0],
                        after.1.position[1],
                        after.1.position[2],
                    );
                    position = start + (end - start) * blend;
                    turn = turn_toward(before.1.facing, after.1.facing, blend);
                }
            }
            transform.translation = position;
            transform.rotation = nalgebra_glm::quat_angle_axis(turn, &Vec3::y());
            velocity.0 = Vec3::new(newest.velocity[0], newest.velocity[1], newest.velocity[2]);
            facing.0 = turn;
            crouch.0 = newest.crouch;
            grounded.0 = newest.grounded;
        },
    );
}

pub fn turn_toward(from: f32, to: f32, blend: f32) -> f32 {
    let mut difference = to - from;
    while difference > std::f32::consts::PI {
        difference -= std::f32::consts::TAU;
    }
    while difference < -std::f32::consts::PI {
        difference += std::f32::consts::TAU;
    }
    from + difference * blend
}
