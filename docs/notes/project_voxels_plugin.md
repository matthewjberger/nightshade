# nightshade-voxels holds the block world, and the game names its own blocks

`crates/plugins/domain/nightshade-voxels` carries what every block game needs and no
block a game invents: the chunk store, the streaming, the digging crew, the
slab mesher, the sight pass, the shadow ring, and the block edits.

`VoxelWorld` holds the block table, the chunks, the surface heights, the dirty
slabs, the markers and the edits. A chunk is `Arc<Vec<u8>>` of block indexes,
its stage, the faces its air joins, and whether it is solid throughout. The
table is the contract: one `BlockLook` for each block index with the atlas
cells of its faces, its shape, whether it is cutout, whether it hides its
neighbors, how full it is if it is a liquid, which neighbor holds it up,
whether it stands two high, and whether the world keeps a marker for it. A
shape is a cube, a cross, a liquid, or a list of parts, and a part is a box
with a texel patch for each face and a lift on its top corners, so a torch, a
signpost and a door are rows in the game's table and the mesher draws what
the row says. Everything the crate reads about a block comes from that table,
so the crate names no block.

`VoxelViewer` is where the world is seen from, which the game writes each
frame, and `VoxelSettings` names the materials the three layers draw with, the
block below the floor, and the radii the streaming and the shadows reach.

A game fills a column through `Dig`. The crate hands out the columns around
the viewer, nearest and ahead first, runs `Dig::dig` on the digging crew over
a private world that already holds the player's edits, finishes each column
the same way, and settles it into the world with `Dig::settle`. The game's
digger keeps its own order and report types, so villages, spilled blocks and
sign texts travel with the column without the crate naming them.

`write_block` puts a block in and keeps the surface and the markers right.
`set_block` is the player's edit: it is kept, the slabs around it are meshed
again, and the block and its neighbors get a tick. The table also says whether
a block is solid, falls and can be replaced, and which liquid a block is and
how full. The table lists the liquids, each with its six level blocks, how far
it runs, how many frames a step takes, and the material its faces draw with,
and every liquid gets a slab layer of its own. A block that falls leaves the
world and drops through the air as a cube of its own, gaining speed until it
meets a block it cannot replace, where it becomes a block again. The crate
runs those falls and every liquid's spread, reports where one liquid met another in the
world's `met` list for the game to resolve, and
carries the raycast, the box tests and the ground heights. A block that loses
its support lands in the world's fallen list, and what drops from it is the
game's to decide. The crate also moves a body through the blocks one axis at
a time and says where it landed, finds the water surface at a place, places a
block by the rules a hand obeys, swings a block to the block the table names
and its tall partner with it, builds a cube mesh in the vertex encoding its
materials read, packs the edits as bytes for a save, and resets the world down
to its table, crews included. The digest of a generated world is the
`world-digest` tool's, which hashes the chunks and the surface with `fnv`.

The settings carry the block size. The slabs are scaled by it and the viewer's
eye is read through it, while every query and command speaks in block
coordinates, and `in_blocks` and `in_world` convert a position either way. A
game of small voxels sets it once and reads its rays through it.

`VoxelsPlugin<D>` inserts the resources, hires the two crews, and registers
the streaming, the settling, the shadows, the sight pass and the mesher. The
game inserts `D::Orders` itself and composes the plugin after its own
systems, so the mesher reads the viewer of the same frame. The plugin splits
the threads the machine leaves the crews, three of every five to the diggers
and the rest to the masons, so the split holds whichever crew is hired first.
A frame settles two columns and places two slabs, whatever the refresh rate
of the display, so the work a frame does for the crews is bounded and the
rest waits on the bench. The sight pass reads the column tops within
the view radius and floods the caves around the chunk the viewer stands in,
so a settled column costs it one entry rather than a pass over every chunk.
That rule fits a terrain, where nothing stands open more than a slab below
its top. A hollow structure taller than a slab keeps its lower storeys hidden
under it, so a world of buildings sets the viewer's `whole` flag and every
slab from the floor to the top is shown.

`apps/features/liquids` is the small proof of the crate over a second table:
water in the low ground, lava on the hot plateaus with its own reach, pace and
glowing material, and stone wherever the two meet.

`apps/features/demolition` is the proof of the block size: a brick building
of voxels one eighth of a meter across, with concrete pillars, wooden floors,
framed glass and a roof, over a flat ground. The camera flies free, a marker
sphere shows where the next blast lands and how large it is, and a click
takes that sphere of blocks out. The bricks and glass above the hole fall
into it through the crate's gravity, while the floors and the roof hold, and
the building can be put back whole.

`apps/games/voxels` is the game over it. It keeps its `BlockKind`, its
generation passes, its items, its interface and its save format, builds the
table from `BlockKind` at startup, and writes the viewer from the player.
