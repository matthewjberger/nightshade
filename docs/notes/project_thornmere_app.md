---
name: project_thornmere_app
description: apps/games/thornmere is the offline MMO, a third-person zone built from the Kenney kits where every other player is a simulation with a task loop and a voice in the zone chat
type: project
---

`apps/games/thornmere` is a fantasy MMO that runs alone. The zone is three
hundred units on a side. The walled city of Thornmere fills the south: a moat
with a stone bridge, a gatehouse, a trade square with a fountain, a statue, and
market stalls, an auction house, a bank, an inn, an apothecary, a canal with
three bridges, a cathedral square with a graveyard, an old town with a smithy
and a stable, a park, a mage tower, and a keep on a raised terrace behind its
own wall. Guards stand at the gates and walk four beats, dogs run two routes,
and horses stand in the stable. Beyond the moat lie the boar field with its
bison, Wolf Wood, Holt's farm, Redharrow Fort with its palisade and bandits,
the barrow with the Sunken Crypt dug into its top, Ironjaw Mine with a
headframe, rails, and a pit the kobolds hold, and Mirror Lake with a harbor of
docks, a moored ship, and boats. Roads join them and the terrain flattens under
the roads, the city plateau, and the pits.

The zone is dressed from the Kenney kits under `apps/games/thornmere/assets`:
the Fantasy Town Kit for the houses, stalls, lanterns, roofs, fountains, and
roads, the Castle Kit for the city walls, gate, bridges, hexagon tower, and
siege engines, the Graveyard Kit for the crypt and the undead, the Mini Dungeon
kit for the crypt walls and floor, the Retro Medieval Kit for the fort
palisade and the mine headframe, the Survival Kit for tents and tools, the
Nature Kit for trees, rocks, crops, and flowers, the Pirate Kit and the
Watercraft Pack for the harbor, the Prototype Kit for the horses, dogs, and
bison, the Food Kit and Mini Market for the stall goods, the Holiday Kit for
the mine rails and carts, the Mini Arena for the soldier that is every guard
and the statue in the square, the Blocky Characters for bandits and kobolds,
and the Cartography Pack for the world map icons. Every model is a
`ModelFile` in `data/assets.rs`, generated from the folder listing and looked
up by file name with `commands::models::model`. A kit that shares a name with
another kit is copied under a prefix, because the lookup is by name alone.
`kit_textures` hands each model every texture its kit carries, so the Retro
and Blocky kits, which reference several textures, load like the rest. The
models load on the prefab crew at startup. A creature or a prop is a root
entity with `Look` and `Fit` from the first frame, and `systems::models::dress`
hangs the prefab under each root on the frame the last model lands or fails,
turned by `MODEL_FACING` and scaled by `Fit`. A character model carries its own
glTF clips, and `animation::pose` picks idle, walk, sprint, sit, attack, or die
from the root's state through the `Rig` handle. The engine samples only the
channels a clip carries, so the pose system resets the rest pose from the
`static` clip at every clip change. An animal carries walk, run, up, and down,
and `pick_clips` maps those onto the same slots. The boars and wolves stay cube
quadrupeds whose legs are `Limb` parts swung from the stride.

The city, the dungeons, and the harbor are built in `commands::city` and
`commands::dungeons` from tables in `data`: `CITY_BLOCKS` places every house
by cell count, floors, wall material, and roof, `CITY_STREETS` lays the road
tiles, `PITS` cuts the crypt and the mine floors, and `GUARD_POSTS`,
`GUARD_ROUTES`, and `DOG_ROUTES` place the watch. A guard on a beat carries
`Beat`, `Post`, and `Pause`, and `guards::patrol` walks it. A townsperson with
no quest is a figure with a plate and nothing else.

Every house has an interior. The front wall is a wide doorway, each storey
has a floor slab, and `commands::interiors::furnish` fills the ground floor
from the Kenney Furniture Kit by the `Interior` the block names: home, inn,
auction house, bank, chapel, keep, smithy, apothecary, guard house, cabin, or
warehouse. Furniture pieces have their origin at a back corner, so a layout
places them by corner in house-local coordinates, x to the right and y toward
the door. Hearths and candles carry a permanent point light. The interface
panels and the world map carry the Kenney fantasy border as a nine-slice
frame, the way gatewarden draws its panels.

Quests carry a story line that the accept banner shows, and an objective that
is a kill count, a gather count, a delivery, or a discovery. `quests::explore`
completes a discovery when the daylight system marks the place discovered.
The world map opens on a key: a picture of the zone painted from the terrain
at startup, with the roads, the city, the moat, the pits, and the water, the
Cartography icons and names on every place, the player as a dot, and a gold
mark for every active quest target. The sky carries drifting cloud clusters
that cast moving shadows, wheeling flocks of birds, and linear distance fog
that carries the hour in its color, over the engine's day and night sky with
its stars and moon. The lit air is off, because a wood of casters turned it
into long bands across the view: refer to
`reference_volumetric_fog_draws_sun_shafts.md`.

The population is the point. Forty-eight simulated players carry a task loop,
rest at the board, walk to the camp their level fits, hunt the mob their hunt
names, walk back to the giver, turn in, and level. They log off at night and
back on in the morning, and the count in the corner follows the hour. The chat
is a scheduler in `resources::ChatLog`: a thread is an opener and up to four
replies, each on its own delay, with placeholders filled from the roster of
online people, the spawn table, the places, and the quests.

Feel lives in three small resources. `Cues` collects sound requests and one
system plays them, with the Kenney audio clips under `assets/audio` and three
synthesized whooshes. `Bursts` collects particle requests and one system spawns
them. `Feel` carries the camera trauma, the hurt flash, the death grey, the
combat timer, and the action bar flashes. Spells are envelope-driven meshes in
`commands::spells`: a `Shape` with an `Age` over a `Span`, so a blast grows and
collapses, a shard rises and melts, a ring expands and thins, and a lamp is a
point light whose intensity follows the same curve. Nothing appears or vanishes
in one frame.

A mob is its components and nothing else: `Aggro`, `Leash`, `Worth`, `Bounty`,
`Rebirth`, `Rank`, `Level`, and the stats every fighter has. `data::SPAWNS` is
a table the builder reads once, and no system reads it. Quests and the hunters
match a mob by the `Name` it carries.

The current target carries the engine stencil outline from
`nightshade_render::resources::outline::Outline`, red for a common mob and
gold for an elite, which draws only with `debug_render` enabled on the wgpu
backend, the way `apps/features/picking` enables it.

Name plates are billboard text entities moved by `plates::follow` to the owner
plus its `Lift`. The lines of a billboard render from the bottom up, so a plate
string puts the top line first. The camera is the third-person camera of
`nightshade-camera-controller` driven by its two functions registered as
systems, and `CameraControllerPlugin` is not composed. `setup::initialize` and
`setup::build_interface` are two startup systems, because `UiBuild` names
`TextState` and a system may not name a resource twice. The interface icons are
one atlas packed from the Kenney Game Icons at build time and uploaded once.

The starter area is Fallowmere Wood, the place that wraps the road north of
the gate: dense round-canopy oaks and firs off the road, a fenced and lamplit
lane, a hamlet with an inn, a house, a barn, and a field at the crossroads,
young wolves and kobold vermin for the first two levels, and an innkeeper with
the first quests. Monsters take a `wild_spot`, which retries a place spot until
it is off the road, out of the city margin, out of the hamlet clearing, and
out of the water, and a mob respawns at that home, so the roads stay clear.

The cursor picks through the GPU pick pass, the same path
`apps/features/picking` takes. `picking::hover` asks for a pick when the
cursor moves or every fifth of a second, matches the returned mesh id to its
entity, and walks the hierarchy up to the first root that carries `Health` or
`Pickup`, which gets the engine outline. `picking::click` turns a hovered
monster into the target, knocks on a quest giver in reach so the giver speaks
again, reports a pickup nobody asked for, and otherwise sets the player's
`Approach`, which `player::run` walks toward until the player is in reach or
touches a movement key. A click on nothing clears the target. Picks pause
while the right button orbits the camera and while the retained UI holds the
pointer.

The game opens on a character creation screen, an app state `Screen::Creation`
over the live world. A menu panel from `nightshade-ui-menu` lists five callings
from `data::CALLINGS`, each with its own Kenney character, health, mana,
strength, speed, six abilities out of the thirty in `data::ABILITIES`, one of
the five Kenney frame borders, and an accent tint that colors every HUD frame.
A stage figure stands at `STAGE_SPOT` under a third-person camera of its own,
and a click on a calling replaces the figure and the sheet. The name field
starts with a rolled name and rolls again on a button. Entering the world
despawns the stage, spawns the player from the calling, and builds the HUD in
`menu::open_hud`, so the HUD is never built before the calling is known. The
systems that touch the player, the quests, and the HUD run under
`while_in(Screen::World)`, and the world simulation runs in both screens.

Models load outward from the camera. `models::demand` keeps a small number
of loads in flight and, each frame, requests the unloaded models whose
nearest undressed root is closest to the active camera. `models::dress`
attaches prefabs to the undressed roots whose kit has landed, nearest first,
a bounded number per frame, and tags each root as dressed, so a root spawned
later, the player among them, is dressed on the next frame. Monsters stop
at a standoff from the player, and `monsters::space` pushes live monsters
apart from each other and out of the player each frame.

The game drives the sun itself in `daylight::turn` with the engine cycle
off. The clock still advances the engine hour, so the sky follows it, but the
sun direction keeps an elevation floor through the day, because a sun on the
horizon threw tree shadows across the whole zone as long dark streaks, and
the night lights the world from a fixed moon at a steep angle.


Gear is the other half of the loot. `data::items` is the table of thirty-seven
items: a name, a grade that colors it, the place it is worn, the draught it
pours, the strength, health and mana it adds, the level it asks for, its worth
in copper, and the Kenney board-game icon it shows. `DROPS` names a prey and an
item with a chance, so `queries::roll_drop` reads the table by the name the mob
carries, the way the quests do, and a kill by the player stows one item in the
bag. The bag is sixteen slots of stacks and the gear is four worn places, both
in `ThornmereResources`, so a save writes them with the rest of the character.
`player::grow` builds max health, max mana and strength from the calling, the
level and the worn gear together, and it compares what it wants with what the
player carries rather than with the plate level, because the plate is relabeled
earlier in the frame.

Four panels hang in `systems::panels`, built once in `menu::open_hud` beside
the HUD and the world map, each hidden until it is asked for: the character
sheet on the left, the bag on the right, the quest log in the middle, and the
merchant where the sheet sits, which pushes the sheet away while it is open. A
slot is a node with `with_interaction`, so `ui_button_clicks` reports it and the
retained UI claims the pointer over it, which is what keeps the world from
taking the same click. A click on a bag slot wears the item, drinks the
draught, or sells the stack when a merchant is open. A click on a worn place
takes the item off. A click on a ware buys one for four times its worth.
`panels::refresh` writes only the rows that changed, and an icon changes by
writing the node's `UiNodeContent` and its tint.

The three merchants in `data::items::MERCHANTS` are figures with a `Merchant`
component, and `picking::click` opens the shop when the player is in reach and
walks there when the player is not. `panels::tend` closes the shop when the
player leaves.

The character is kept. `data::save::Save` carries the name, the calling, the
level, the experience, the kills, the purse, the hour, the spot, the bag, the
worn gear, the quests and the places found, and `systems::save::keep` writes it
through `nightshade-save` every twenty seconds. The creation screen reads the
file at build time, and when there is one it shows a button that returns the
character: `menu::choose` takes the calling and the name from the file and
`menu::enter_world` restores the rest onto the spawned player. A new character
starts with a shortsword, a tunic and three rations.
