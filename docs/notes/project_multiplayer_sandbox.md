---
name: project_multiplayer_sandbox
description: apps/games/multiplayer/sandbox is the shared sandbox, where the host plays and serves or serves alone, a guest sends only input, and nightshade-net carries the messages
type: project
---

`apps/games/multiplayer/sandbox` is the first app in this repository that talks
to another machine. Two or more copies share one world. The copy that starts
first serves and plays, and each later copy joins it with `--join <address>`.

```
just run multiplayer_sandbox
just run multiplayer_sandbox --join 127.0.0.1
just run multiplayer_sandbox --serve
```

With no flag the app serves on port 7777 and plays, so one copy is a working
world on its own. `--host <port>` moves the port, `--name` sets the name the
other players see, `--play` opens in the world rather than on the title screen,
and `--wander` walks the character in a circle with nobody at the keyboard,
which is how two copies are watched side by side. `--serve` runs the world with
no window at all, which is the dedicated server, and every copy that joins it is
a guest.

## The model

One binary is both the player and the server. The host simulates every body and
sends each guest a snapshot on a fixed tick of 30 Hz from an accumulator over
`Time.delta_time`. A guest sends its walk direction and nothing else, so a guest
cannot move faster than the rules permit or stand inside a wall, because it
never sends a position.

`--serve` composes the same game over the platform, the scene, the graphics
description and the physics, with no window plugin and no renderer, and sets a
runner of its own that steps the schedules on a 30 Hz clock it writes into
`Time` itself. Nothing in the simulation asks what is on the screen, so the
server runs the systems the player runs, less the ones that draw.

A body this machine simulates carries `Steering` and `Wants`. A body someone
else drives carries `Track`, the snapshots that arrived for it. Nothing asks
what kind a body is: the step system names `Steering`, the follow system names
`Track`, and each selects what it can work on. On the host every body carries
`Steering`, and on a guest only the player at that keyboard does.

`queries::step::step_character` is the one movement function. It reads the level
through `nightshade-physics` and nothing else, so the host, a guest's own
prediction, and the replay after a correction all reach the same answer from the
same inputs. It is a lifter spring over a five point ground ray and a bumper
capsule that depenetrates in three passes, lifted from `apps/games/sandbox`.

A snapshot carries only what changed since the last snapshot that guest was
sent, and only what stands inside `INTEREST_RADIUS` of that guest's own body. A
guest that hears nothing about a body keeps the last it was told, so a far body
holds still rather than going away. The snapshots that arrive in one frame are
merged rather than the newest taken and the rest thrown away, because each one
carries a part of the world and not the whole of it.

A guest draws every other player a tenth of a second in the past, between the
two snapshots that bracket that time. It draws itself from its own prediction.
The prediction advances every frame while an input is saved once a tick, so a
replay lands near the prediction rather than on it, and `reconcile` walks a
small error in rather than taking it at once. An error past `CORRECTION_SNAP`
means the host refused a move, and that one is taken at once.

## Who shot whom

A player fires with the left mouse button, and the shot travels as the eye
position and the eye direction rather than as a claim about what it hit. A guest
names the tick it was drawing, which is the interpolation delay behind the tick
the host is on, and the host puts the world back to that tick out of the last
`HISTORY_TICKS` snapshots it sent before it answers. Without that rewind a
player on a long link would have to lead every shot by the length of its own
link.

A tag that lands leaves a mark where it landed. The mark is the general case of
a replicated thing: the host holds the list, gives each one a number and a kind,
and sends a spawn when it appears and a despawn when its life runs out. A guest
that joins later is told the list in its welcome, so it sees the marks that are
still standing. The kind names a row of a table both machines carry, so the wire
never names a model.

## What the transport is

`crates/plugins/domain/nightshade-net` carries the messages. It is a topic broker and
its clients over TCP and UDP, and it names no engine type, so it takes the shape of
`nightshade-worker`: no plugin, no system, and no resource of its own. The app
owns the thread. `commands::net::open_wire` starts one thread with a
current-thread runtime, and two unbounded queues are the only things the frame
touches.

The broker writes the name of the peer that published into every frame it
delivers, so a receiver reads who spoke from the frame rather than from anything
the message claims. A guest publishes every request to `sandbox/request` and the
host reads the peer off `Message::sender`. The host announces to everyone on
`sandbox/state` and speaks to one player on `sandbox/to/<peer>`, which carries
that player's welcome and its own snapshots. A departure arrives on the broker's
own `net/peers/disconnected`, and the peer is read back out of the client
identity, which is `p<peer>_<uuid>`.

The crate also speaks UDP, where each publish names its own guarantee. An
unreliable publish is sent once, a sequenced one is dropped when a newer one has
already arrived, and a reliable ordered one is retransmitted until it is
acknowledged and delivered in order. Fragments, an acknowledgement history and a
congestion window are in `reliability.rs`, and `Broker::start_udp_listener` puts
datagram peers on the same subscription table as the stream peers. The sandbox
itself still joins over TCP.

A guest greets the host on every tick until it is seated, and the host answers
every greeting rather than only the first. One lost welcome would otherwise
leave that guest greeting a host that never replies.

## What it is made of

The players are the eighteen Kenney Blocky Characters, each with its own clips,
so every peer that joins wears a different body and no art is tinted. The course
is the Kenney Prototype Kit: ramps, stairs, walls with a doorway and a window,
columns, crates, and markers. The animals walk rings from the same kit. Both
kits sit under `assets/models` and are embedded with `include_bytes!` from the
table in `data/assets.rs`, which is generated from the folder listing.

The engine samples only the channels a clip carries, so `systems::animation`
resets the rest pose from the `static` clip at each clip change.

## What it does not do

The app leaves `just lint-wasm`, because a browser has no TCP socket and this
repository refuses a stub that pretends otherwise. It carries no `Trunk.toml`
and no `index.html`.
