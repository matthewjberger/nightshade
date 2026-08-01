# nightshade-voxels

A streamed block world. The crate holds the blocks in chunks, streams columns in and out around a viewer, meshes the dirty slabs on a worker crew, and places the finished meshes as scene entities. Gravity, liquid spread, shadow casting and slab visibility run over the same storage, and the game supplies the terrain itself through the `Dig` trait.

## Composing it

```rust
app.insert_resource(Plan);
app.add_system(Stage::Startup, (systems::setup::initialize, systems::interface::build));
app.add_plugin(VoxelsPlugin::<Digging>::default());
```

`VoxelsPlugin<D>` inserts `VoxelWorld`, `Meshes`, `VoxelViewer`, `VoxelSettings`, `Streaming`, `Sight` and `Falling`, and hires two worker crews, `Workers<ColumnJob<D>>` for the terrain and `Workers<Ground>` for the meshing, splitting the free threads between them. Its nine systems, streaming, settling, shadows, sight, gravity, falling, liquids, mesh rebuilding and mesh placement, all run on `Stage::Update`.

## Using it

```rust
impl Dig for Digging {
    type Orders = Plan;
    type Order = ();
    type Tools = ();
    type Report = ();

    fn order(_orders: &mut Plan, _column: ColumnKey) {}

    fn dig(_order: (), _tools: &mut (), world: &mut VoxelWorld, column: ColumnKey) {
        write_block(world, IVec3::new(x, y, z), Block::Ground.index());
    }
}
```

`Dig` is what the game implements to fill a column. `order` reads whatever the game keeps in its `Orders` resource on the main thread, `dig` runs on a digger thread and writes the blocks with `write_block`, and `settle` runs back on the main thread and names the columns that now need another pass. Outside generation, a system takes `ResMut<VoxelWorld>` and calls `set_block`, `place_block`, `swing_the_block` or `reset_the_world`, and reads the world with `block_at`, `raycast`, `column_top` and the key helpers in `queries`. `in_blocks` and `in_world` convert between world coordinates and block coordinates through `VoxelSettings`, whose `block_size` is the scale the whole world is drawn at.

## Taking it from another crate

The crate splits at a `contract` feature.

```toml
nightshade-voxels = { path = "../nightshade-voxels", default-features = false, features = ["contract"] }
```

The contract carries the block vocabulary and nothing that runs: `Blocks`, `BlockLook`, `Shape`, `Part`, `Patch`, `Face`, `Fill`, `Liquid`, `Slide`, `Hit`, `Edits`, the `ChunkKey`, `ColumnKey`, `SlabKey` and `Cell` keys, `ChunkLayer`, and the `AIR` and `FULL_LEVEL` constants.

## Shapes

A block is a `Cube`, a `Cross`, a `Liquid` or `Parts`. A block of parts is a list of boxes in its table row, each with a texel patch of the block's cell for its sides, its top and its bottom, and a lift that moves its top corners, which is how a torch on a wall leans. A block that stands two high sets `tall`, so it is placed, kept and swung as a pair, and `stacked` names the cell its upper half draws. A block that hangs on a neighbor sets `support` to the direction of that neighbor, and it falls out of the world when the neighbor goes. The crate reads all of this from the table and names no block.
