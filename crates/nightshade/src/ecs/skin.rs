//! Skeletal animation skinning for mesh deformation.
//!
//! Bind meshes to skeletons for GPU-accelerated vertex skinning:
//!
//! - [`Skin`]: Binding between a mesh and its skeleton joints
//! - [`Joint`]: Individual bone in the skeleton hierarchy
//!
//! Skinning is handled automatically when loading glTF models with skeletons.
//! The animation system updates joint transforms, and the GPU computes
//! final vertex positions using inverse bind matrices.
//!
//! # How Skinning Works
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │  1. Skeleton hierarchy defines bone relationships               │
//! │     Root → Spine → Chest → Head                                 │
//! │                  └→ LeftArm → LeftHand                          │
//! │                  └→ RightArm → RightHand                        │
//! │                                                                 │
//! │  2. Each vertex has up to 4 joint influences + weights          │
//! │     vertex[i].joints = [spine, chest, 0, 0]                     │
//! │     vertex[i].weights = [0.3, 0.7, 0.0, 0.0]                    │
//! │                                                                 │
//! │  3. GPU computes: final_pos = Σ (weight[j] × joint_matrix[j])   │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Automatic Setup (glTF)
//!
//! glTF models with skins are automatically configured:
//!
//! ```ignore
//! // Load skinned model
//! let prefab = import_gltf_from_bytes(world, character_bytes, "character")?;
//! let entity = spawn_prefab_with_animations(world, &prefab, Vec3::zeros());
//!
//! // Skin component is already attached and configured
//! // Just play animations - skinning happens automatically
//! if let Some(player) = world.get_animation_player_mut(entity) {
//!     player.play(WALK_ANIMATION);
//! }
//! ```
//!
//! # Manual Skin Setup
//!
//! For procedural skeletons:
//!
//! ```ignore
//! // Create joint entities
//! let root = world.spawn_entities(JOINT | LOCAL_TRANSFORM | GLOBAL_TRANSFORM, 1)[0];
//! let spine = world.spawn_entities(JOINT | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | PARENT, 1)[0];
//! let head = world.spawn_entities(JOINT | LOCAL_TRANSFORM | GLOBAL_TRANSFORM | PARENT, 1)[0];
//!
//! // Set up hierarchy
//! world.set_parent(spine, Parent(root));
//! world.set_parent(head, Parent(spine));
//!
//! // Configure joints
//! world.set_joint(root, Joint::new(0).with_name("root"));
//! world.set_joint(spine, Joint::new(1).with_name("spine"));
//! world.set_joint(head, Joint::new(2).with_name("head"));
//!
//! // Create skin with inverse bind matrices
//! let skin = Skin::new(
//!     vec![root, spine, head],
//!     vec![
//!         Mat4::identity(),           // root inverse bind
//!         Mat4::new_translation(&Vec3::new(0.0, -1.0, 0.0)),  // spine
//!         Mat4::new_translation(&Vec3::new(0.0, -2.0, 0.0)),  // head
//!     ],
//! ).with_name("character_skin");
//!
//! // Attach skin to mesh entity
//! world.set_skin(mesh_entity, skin);
//! ```
//!
//! # Skin Properties
//!
//! | Field | Description |
//! |-------|-------------|
//! | `joints` | Ordered list of joint entities |
//! | `inverse_bind_matrices` | Transform from mesh space to bone space |
//! | `name` | Optional identifier |
//!
//! # Joint Properties
//!
//! | Field | Description |
//! |-------|-------------|
//! | `index_in_skin` | Index into the skin's joint array |
//! | `name` | Optional bone name (for animation targeting) |
//!
//! # GPU Skinning Pipeline
//!
//! 1. Animation system updates joint `LocalTransform` components
//! 2. Transform propagation computes `GlobalTransform` for each joint
//! 3. Compute shader calculates joint matrices: `global × inverse_bind`
//! 4. Vertex shader applies weighted joint influences to each vertex
//!
//! # Performance Notes
//!
//! - Skinning runs entirely on GPU via compute shaders
//! - Up to 4 joint influences per vertex (glTF standard)
//! - Joint matrices uploaded once per frame per skin
//! - Multiple meshes can share a single skin

pub mod components;
pub mod systems;

pub use components::*;
pub use systems::*;
