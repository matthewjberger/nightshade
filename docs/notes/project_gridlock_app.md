# apps/games/gridlock is the streamed city on the Kenney city kits

`apps/games/gridlock` is an open city at night, built from the City Kit roads,
the commercial, industrial and suburban kits, the Car Kit and the Mini
Characters, all taken from `nightshade-kit` behind their features. You start on
foot on a downtown sidewalk, and any car in reach is yours.

## The grid

The world is a grid of cells of eight tiles, and a tile is eight meters, so a
cell is 64 meters. Every cell carries a road along its west and south edges,
with the crossroad tile at its corner and a crosswalk tile on each approach,
so the roads are a pure function of the cell key and no cell needs its
neighbor. The road tile of the City Kit runs along local X, and a lamp post's
arm reaches along local minus Z, which is what the yaws in the layout job
follow. The tile scale sets everything else: a car is the Car Kit at 1.4, which
fits the lane, and a person is the Mini Character at 2.6.

The district comes from the cell's distance to the origin: a downtown core,
a commercial ring, an industrial band to the east, and the suburbs beyond.
Downtown puts one tower at the block center, scaled three tiles, with shops
around its feet. The block interior is a courtyard: parking, planters and trees
in midtown, chimneys, tanks, trucks and cones in the works, trees and a fence in
the rows.

## The street

The road tile of the City Kit is one tile of asphalt with a curb strip one
tenth of the tile wide on each side, so at eight meters a tile the sidewalk on
the tile is under a meter and the asphalt is 6.4 meters. `pave` lays a slab
of `PAVEMENT` width beside each curb, flush with the curb top, and the block
origin and span step in by that width, so the buildings, the props and the
doors stand at the slab's inner edge. The asphalt is `ROAD_LIFT` below the
sidewalk, and `surface_height` says which a point stands on, so the traffic
rides at road level, the hero and the walkers step down to cross, and the
player's car climbs the curb. `SIDEWALK` puts the walkers' corners on the slab
inside the crossing tile's stripes.

## The streaming

`systems/layout.rs` is the worker job. `stream::feed` keeps the anchor's cell
until the anchor leaves a band around that cell, the deadzone in the settings,
so a walk along a cell boundary never flips the center back and forth, and
each flip is what settles and tears cells down. It recomputes the wanted disc
only when the center moves or the camera turns a sector, hands cells to the
crew nearest and ahead first, and lists the cells outside the keep radius. The
roots settled a frame, the roots unloaded a frame, the model imports taken a
frame and the prefabs dressed a frame are settings, because those budgets are
what a hitch is made of. `stream::settle` puts a landed layout into the city
at once, so its lots and doors work, and spawns its roots from a queue a
budget at a time, bare roots carrying `Look`, `Fit` and `Cell`, with the
lamps, the neon strips and the pavement slabs after the last root. The models
system dresses them nearest the camera first as each kit lands, and
`models::collect` takes only a few imports a frame, because an import is a
mesh and texture upload. `stream::unload` drops a leaving cell from the city
at once and moves everything that carries its `Cell` onto a dying list, the
people and the cars of that cell included, because those write their `Cell` as
they move, and despawns a budget of them a frame.

A root carries `Look` and `Fit` only until it is dressed. `models::dress`
takes the two off with the prefab child in place, so the demand and the dress
passes walk the few hundred roots that still wait rather than every root in
the city, and a prop that lost its `Look` is still a prop by `Cell` and
`Prop`. The engine's frame cleanup reclaims a mesh, a texture or a material
that no live entity names, and a prefab that spawns afterwards logs an error
and draws nothing, so `models::collect` protects every mesh and texture of an
import as it lands. A kit model that streams out of view and back in keeps its
geometry that way.

The walkers spawn spread along a sidewalk segment rather than at its corner,
because the populate pass spawns one a frame at one of four corners and eight
frames stacked eight people on the same spot. They also push each other apart
within a few radii, the way they push off the cars and the hero.

## The people and the traffic

The people and the cars are simulated only inside a life radius of cells
around the anchor, which is the player, the car, or the fly camera while it
flies. The populate systems skip the cells beyond it, and a walker or a car
that finds its cell beyond it plus one despawns itself. The loaded city beyond
that radius is placements and lights and nothing moves in it.

A walker moves between the four sidewalk corners of each intersection, along
the sidewalks and across the crosswalks, so the graph is the intersection key
plus a corner index and needs no data. Traffic drives the right lane at a
constant lateral offset, decides its next turn at each intersection, and takes
a turn along a quadratic curve from the lane it leaves to the lane it enters.
The light at an intersection is a phase of the uptime salted by the key, green
for one axis at a time with a short all-red gap.

## The tuning

The handling, the walk, the traffic, the crowd, the night, the cameras and the
city plan are a `config!` declaration in `data/settings.rs`, read through
`Config<Settings>` and kept in `gridlock/settings.toml`. The pause screen
carries the panel `nightshade-ui-config` builds from it, `tuning::apply`
writes the render look and the walk camera whenever the revision moves, and
the systems that move things read their numbers each frame. The layout job
takes its share as a `Plan` value at submit time, because a worker reads no
resource. The scale of the grid, the collision radii and the tables of model
names stay constants, because the geometry of the kits does not change.

The walk camera runs with the controller's collision off. Its sweep reads
`CameraCollisionHook`, which only `CameraControllerPlugin` inserts, and that
plugin is not composed because it would drive every camera. The app pulls the
camera in itself: `camera::keep_out_of_the_lots` walks the boom from the head
and stops it before the first lot box, the way hollowmere walks its voxel
raycast, and `camera::first_person` hides the hero and its descendants when
the boom drops under the first person distance, with the controller's
`distance_min` at zero so the wheel goes all the way in.

The fly mode is a third camera. `fly::toggle` copies the active camera's
transform onto it and takes its yaw and pitch from the rotation, and
`fly::run` moves it with the keyboard, the mouse and the gamepad. Every system
that moves the hero or the car returns while the camera flies, and the stream
anchors on the fly camera.

Every read of a key or a pad button goes through `nightshade-input`: `Ask` in
`data/ask.rs` names the walk, the sprint, the use, the fly toggle, the steer,
the drive, the handbrake, the rise, the look and the menu, `InputPlugin::<Ask>`
keeps the bindings in `gridlock/bindings.toml`, and the systems take
`Res<Actions<Ask>>`. The drive is one signed axis, throttle positive and brake
negative, from the keys, the arrows, the triggers and the face buttons, the way
hairpin declares it. The fly camera's mouse look stays raw, because the layer
has no shape for a pointer delta. The hero is hidden while the car carries
her, through the same visibility walk that hides her in first person.

## The props

A cell keeps two kinds of footprint. `lots` are the buildings and the parked
cars, which nothing moves. `props` are the lamp posts and the dressing from
the furniture, retro urban and road kits, each an `Option<Lot>` that the layout
job fills and that `props::knock` empties when the car takes the prop down.
A placement that is a prop carries the index, and `stream::settle` puts it on
the entity as `Prop`, on the lamp's light as well as on its post, so one query
by cell and index finds everything that belongs to the post. `drive::update`
tests the car's circle against the standing props: above the knock speed the
prop goes on the `Knocks` list and is left out of the push, below it the prop
pushes the car back like a wall. `props::knock` takes the lot out of the cell,
works the pivot as the lot edge on the far side of the travel direction, and
puts `Toppling`, `Pivot` and `Fallen` on the entity. `props::topple` turns the
transform about that pivot by a smooth step of the tilt and slides the pivot
along, so a corner-origin furniture model and a centered urban model fall the
same way. The walk pushes against standing props too, and the camera boom
ignores them, because a bench would pull the camera into the hero's head.

The footprint table `data/footprints.rs` is generated from the GLB bounds of
every kit the app takes, so `lot_of` and `origin_for` read the real local box.
The furniture kit puts its origin at a corner, the other kits at the bottom
center, and both go through the same quarter turn of the local center, which
is why the layout places a model by the center it wants and asks `origin_for`
for the position to spawn at.

## The rooms

A ring building writes a `Door` on its street face with a room kind by
district. `interiors::enter` builds the room at `ROOM_SITE`, three hundred
meters below the city, from the Building Kit floor, walls, a doorway and a
column at each corner, sized from the building's footprint in wall tiles, and
furnishes it along the walls and across the floor with the rejection test the
courtyards use. Every piece carries `Indoor` with the visit serial, so leaving
despawns that room and nothing else. The hero is moved with the third person
camera's focus and yaw set in the same call, because the controller lerps the
focus and would otherwise fly the camera down three hundred meters. While a
visit is on, `stream::feed` holds the center, `weather::rain` leaves the
emitter where it was, and the walk and the camera read the room's wall and
furniture lots instead of the city's.

## The numbers

`Flashes` is a pool of billboard texts the way hollowmere's floating pool is,
spawned once at startup, and a streak that the combo window resets. A person
hit at speed and a prop knocked over both call `score_hit`, which multiplies
the base by the growth for each step of the streak, picks the word, the color
and the pop for that step, and queues the flash over the spot.
