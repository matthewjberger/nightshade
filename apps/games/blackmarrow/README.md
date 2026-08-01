# blackmarrow

A first-person survival horror game in a pumping station cut into rock under the ice. It stands beside Penumbra: Overture and Amnesia: The Dark Descent.

## What your hands do

You do not press a key to open a door. You take hold of the handle and pull, and the door swings on its hinge at the speed you pull it. A drawer slides out as far as you drag it. The same hold moves a crate, a chair and a spare cell, and a second button throws what you hold.

One raycast from the eye finds what your hand is on. One impulse at the grab point moves it. A door is a dynamic leaf on a revolute joint with symmetric limits, a drawer is a tray on a prismatic joint with a travel limit, and a loose object gets the same pull with gravity taken off it. The joint decides how each one can move, so there is one mechanic and not three.

## The station

Six hand-placed rooms in `data/rooms.rs`: an entry hall, a corridor spine, a pump room, a store, an office and the shaft head. Walls are runs that split around the doorways on them, and every run, floor and ceiling is one static collider drawn as flat quads, 1 m tiles on one grid over each visible face and nothing inside, so the affine warp of the playstation look stays small and no inner face shows through a seam when the vertex snap moves an edge. The texture carries across the seams by an offset in each tile's material. Each doorway carries a hinge side and a lock. The shaft door is locked, and its key is in a drawer in the store.

The entry hall is the last shift's changing room. A caged lamp hangs from the ceiling on a spherical joint and swings when you push it or walk into it, and it casts the shadows of its own bars. A tool rack on the east wall holds a hammer, a wrench, a length of pipe, a crowbar and a glowstick, each a rigid body you can take hold of, carry and throw. The glowstick is emissive and carries its own point light, so it lights whatever you hold it up to. A desk on the west wall holds the last page of the shift log. Reach at it and the page opens on screen, and it goes back down when you put it down.

`data/route.rs` carries the fifteen waypoints the wolf walks between.

## The wolf

It hunts by sight and by sound. It cannot be fought. The model carries no texture coordinates, so its four flat materials are darkened and roughened after the spawn to read as hide under the lamps. A raycast from its head to yours decides whether it can see you, and the `Noises` bus carries every footfall, every thrown object and every door you let go of too fast. Crouch to make less noise, and put the torch out.

## The torch

A cell that drains, a beam that browns out below the low mark, and a spare cell you can find and put in. The beam casts shadows and scatters in the air.

## Running it

```
just run blackmarrow
```

`--straight-in` opens a run without the title screen, and `--with-key` starts you carrying the shaft key.

The settings live in `blackmarrow.toml` under the `nightshade` data folder through `nightshade-config`. `Invulnerable` is on by default, so the wolf still catches you, latches and shakes the screen, and takes no health. Set it off in the file to play for keeps.

The backquote key opens the developer console from `nightshade-shell`. `noclip` lets you pass through the station, and `noclip 12` sets how fast you fly. While it is on, the keys that walk you also lift you and lower you, and holding shift doubles the speed. `noclip` again puts you back on your feet. `god` flips `Invulnerable` and writes it to the settings file, so it holds for the next run as well. While the console is open the game takes no keys.

Mouse to look, WASD to walk, Shift to run, Ctrl to crouch, F for the torch, hold the left mouse button to take hold, the right mouse button to throw, the wheel to push what you hold away or pull it near, and E to pocket what your hand is on.

## Assets

Textures are the Screaming Brain Studios horror pack, the props are the Kenney furniture kit, and the wolf is from Quaternius. All three are CC0, and each directory under `assets/` carries its licence.
