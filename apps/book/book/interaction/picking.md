# Picking

Picking turns a point on screen into an entity in the world. The maths is a
ray, and the ray is built from the camera the pointer is over.

```demo
id: picking_table
title: Pick a shape
caption: Click a shape to raise it. The ray is built from this figure's own camera and its own rectangle on screen, not from the window.
```

## Building the ray

A camera gives you an origin and a basis. A pointer position inside the
camera's rectangle gives you the two offsets:

```rust
let half_height = (perspective.y_fov_rad * 0.5).tan();
let half_width = half_height * aspect;

let x = (pointer.x / size.x) * 2.0 - 1.0;
let y = 1.0 - (pointer.y / size.y) * 2.0;

let direction = (forward + right * x * half_width + up * y * half_height).normalize();
```

The rectangle matters. A camera that draws to a tile inside the window has its
own size and its own origin, and a ray built from the window instead lands
somewhere else entirely.

## Testing against a sphere

The cheapest useful test is a ray against a bounding sphere:

```rust
let offset = origin - center;
let b = offset.dot(&direction);
let c = offset.dot(&offset) - radius * radius;
let discriminant = b * b - c;

if discriminant >= 0.0 {
    let distance = -b - discriminant.sqrt();
    if distance > 0.0 {
        // hit
    }
}
```

Keep the nearest hit rather than the first. Iteration order over a world is
archetype order, and archetype order has nothing to do with depth.

## Marking what was picked

Put the result on the entity:

```rust
#[derive(Clone, Copy, Default)]
pub struct Pickable {
    pub radius: f32,
    pub picked: bool,
}
```

Then a system reads the flag and changes the material, spawns an outline, or
starts a drag. The pick and the reaction to the pick are two systems, and
neither knows about the other.

## The picking crate

`nightshade-picking` does the same thing against real triangles rather than
bounding spheres, with a BVH behind it. It publishes its results as a
contract, which is how `nightshade-physics` reads trimesh ray hits without
depending on the picking plugin.

Reach for it when a bounding sphere is too coarse. For a handful of round
objects, the twelve lines above are the whole feature.

## Precision

A hit test in a tone-mapped, anti-aliased image is not pixel exact, and it
does not need to be. Give small targets a bounding radius larger than they
look. A pick that lands a pixel outside the shape reads as responsive, and one
that misses by a pixel reads as broken.
