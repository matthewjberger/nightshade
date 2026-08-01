# Notes

One fact to a file: what this engine does, and what it does to you when you get
it wrong. Read the notes that touch what you are about to change, before you
change it. Each one names a fault that is expensive to find on your own.

A note says what is there now and how to work with it. It never records what
the engine used to be, what was tried, or what is planned, because a reader
cannot tell that apart from the truth. Correct a note that is wrong, and delete
one whose subject is gone. Write a new one when you learn something durable,
and give it a line in the list below.

`CLAUDE.md` carries the rules the work runs under and the shape of the engine.
These notes carry what does not fit there.


## The apps

- [apps/games/ashfall is the PS1-style survival horror app, whose two-layer town map holds to four reachability rules](project_ashfall_app.md)
- [apps/book is the mdbook-style book drawn by the retained UI, with live demos and a capture flag for checking any page](project_book_app.md)
- [apps/games/boomstick is the boomer shooter, six hand-authored foundry floors with keyed doors, secrets, four guns and a rocket jump](project_boomstick_app.md)
- [apps/games/coinfall is the Kenney pixel platformer whose clock always falls and whose coins buy it back](project_coinfall_app.md)
- [apps/games/blackmarrow is the Penumbra-style indoor horror app, where one hold-and-pull mechanic drives loose objects, hinged doors and sliding drawers through joints](project_blackmarrow_app.md)
- [apps/games/bulwark is the isometric tower defense on the Kenney tower defense pack, where a four bit neighbour mask picks the road tile](project_bulwark_app.md)
- [apps/games/crosstown is the HD-2D beat-em-up, Kenney Toon Characters on a generated block street](project_crosstown_app.md)
- [apps/games/gridlock is the streamed cyberpunk city on the Kenney city kits, where a worker crew lays each block and every car is yours to take](project_gridlock_app.md)
- [apps/games/hairpin, the arcade rally with drift and lap times](project_hairpin_app.md)
- [apps/games/hearthside is the life sim on the Kenney furniture kit, where a placed piece feeds one need and the wage pays for the next piece](project_hearthside_app.md)
- [apps/games/multiplayer/sandbox is the shared sandbox over TCP, where the host plays and serves and a guest sends only input](project_multiplayer_sandbox.md)
- [apps/games/neon_static, the Hotline Miami style top-down shooter on nightshade-sprite](project_neon_static_app.md)
- [apps/games/nightwire is the interactive fiction app that is retained UI only, with a toggle-able CRT pass](project_nightwire_app.md)
- [apps/games/roundwood is the Animal Crossing style cylinder village](project_roundwood_app.md)
- [apps/games/srpg is the tactics campaign (FFT plus Fire Emblem Three Houses) on two Kenney sheets packed into one atlas](project_srpg_app.md)
- [apps/games/thornmere is the offline MMO, a third-person zone built from the Kenney kits where every other player is a simulation with a task loop and a voice in the zone chat](project_thornmere_app.md)
- [apps/games/voxels, the block world with a main menu and a world plan the player sets before the world is built](project_voxels_app.md)
- [The description language's provided names live in one table, and a new capability is a facility plus a bind pass](reference_description_facilities.md)
- [What the description language promises, and what the interpreter may choose](reference_description_contract.md)
- [A system in the description language asks, and the phase boundary answers](reference_description_commands.md)

## The engine

- [nightshade-config holds the settings an app declares as one enum, and nightshade-ui-config builds the panel that edits them](project_config_crates.md)
- [nightshade-input is the only input layer, and a game never reads a key by name](project_input_actions_one_layer.md)
- [nightshade-save holds the slot, the header, the version and the autosave clock around a game's own save payload](project_save_crate.md)
- [nightshade-kit holds the models, sounds and icons a game builds its world from, the catalog is written from the folders under assets/kits/models, and each kit is a cargo feature](project_kit_crate.md)
- [InputCapture arbitrates the pointer and the keyboard through claims keyed by the type that makes them](project_input_capture_resource.md)
- [Every app loads a model through nightshade-prefab's Loading resource, and the read and the parse run on a nightshade-worker crew rather than on the frame](project_prefab_loader.md)
- [A crew on every spare logical processor starves the frame, so the crews together take one in two](reference_a_full_crew_starves_the_frame.md)
- [The default entity is a live entity, so a handle compared with it reads the first entity as missing](reference_default_entity_is_a_live_entity.md)
- [The retained UI reaches the headset as an OpenXR quad layer through a renderer paint target that redirects the paint passes, and the paint prepare consumes its draw list, so a second paint run in one frame draws nothing](project_openxr_retained_ui_quad_layer.md)
- [The Vulkan backend carries a probe grid for bounced light, an SVGF denoiser, and threaded scope recording](project_vulkan_probes_denoiser_threads.md)
- [nightshade-portal holds the portal capability, and the backend draws the openings from the frame it publishes](project_portal_plugin.md)
- [A ClipSet moves clips from one character to another whose rest pose already matches, and reports every joint it could not find](reference_clip_set_swaps_a_matching_rig.md)
- [A Mixamo clip's forward travel sits in the hips bone's local Y after a Blender import, so a clip that looks in place still moves the character](reference_mixamo_root_travel_hides_in_local_y.md)
- [nightshade-voxels holds the block world, the streaming, the crews, the mesher and the liquids, and a game fills a block table and digs its columns through Dig](project_voxels_plugin.md)
- [A contract feature that gates nothing hides a behavior edge, so the function belongs beside the data it changes](reference_a_contract_must_gate_something.md)
- [Six framework crates carry what makes a horror game feel like one, and a game composes only the ones it wants](project_horror_crates.md)

- [A plugin needs resources it never names, so the crate graph is a DAG and the plugin graph is a hub](reference_a_plugin_needs_what_it_never_names.md)

## Writing a system

- [A parameter system must list Res, ResMut, Commands, and other extract params first and every Query last, or the IntoSystem impl does not match and the error names only SystemCollection](reference_system_params_queries_last.md)
- [A query call inside another query's closure panics with a RefCell already borrowed message, so gather into a Vec first](reference_no_nested_queries.md)
- [A &mut element in a Query stamps a change tick on every visited row, so a wide &mut query makes the renderer resync thousands of entities each frame](reference_mut_query_element_marks_every_row.md)
- [A system on a rayon thread must find every storage cache it reads already filled, because a miss inserts into a map its batch mates share and the heap corrupts at startup](reference_a_parallel_batch_never_writes_a_storage_cache.md)
- [Edge-triggered input is cleared inside the frame schedule, so code that runs outside run_systems reads it already cleared](reference_input_reset_in_frame_schedule.md)
- [A close request from the window sets Window.close_requested and runs one more frame before the loop exits, so a save on exit reads that flag and not should_exit](reference_close_request_runs_one_frame.md)
- [The android input queue has one reader, so nightshade drains it from new_events and winit sees nothing](reference_android_input_queue_has_one_reader.md)
- [A window call from a parameter system deadlocks the frame, so a system that calls the window names the MainThread parameter](reference_window_calls_need_the_main_thread.md)
- [A plugin's on_pre_render hook fires before render_frame, so anything it hands the renderer must already be complete](reference_pre_render_runs_before_render_frame.md)
- [A workspace build unifies features and hides per-app faults, so check each app on its own](reference_check_each_app_alone.md)
- [A game copied from the template does not quit on escape unless it registers escape_key_exit_system itself](reference_template_escape_exit_trap.md)
- [A camera parented to a character body moves in whole physics steps, so a fast first person game sets the step from the frame and leaves interpolation off](reference_camera_on_a_body_inherits_the_physics_step.md)
- [CameraControllerPlugin drives whichever camera is active, and a transition into the current state fires no on_enter](reference_active_camera_and_screen_states.md)
- [day_night_cycle_system is a no-op unless auto_cycle is true AND atmosphere is Atmosphere::DayNight, and it needs sun_entity set](reference_day_night_system_gating.md)

## Drawing

- [Nightshade renders reverse-Z, so a new pass clears depth to zero and compares with GreaterEqual, and getting it backwards makes geometry look translucent](reference_engine_renders_reverse_z.md)
- [A material an app registers is reclaimed unless it is held, so a mesh that names it draws white and untextured](reference_app_material_must_be_held.md)
- [A textured material on a spawn_mesh_at mesh renders untextured, because only the instanced path carries the texture](reference_textured_material_needs_instanced_mesh.md)
- [An edit to one material each frame syncs every material each frame, so a scrolling picture drifts on its texture transform](reference_material_edit_per_frame_syncs_every_material.md)
- [A render sync pulled out of scene_sync must gate on MeshRenderState transform_dirty as well as component change ticks, or moving entities freeze](reference_extracted_sync_needs_transform_dirty.md)
- [Every light consumer in the renderer reads the world, and one that does not makes shadows vanish without a word](reference_every_light_consumer_reads_the_world.md)
- [Volumetric fog lights a grid against the sun cascades, so every caster throws a shaft across the view, and a density of zero is not off](reference_volumetric_fog_draws_sun_shafts.md)
- [Volumetric fog occlusion-tests only the first directional light, and skips it when shadows are off, so a fill light washes a sealed interior](reference_volumetric_fog_lets_a_shadowless_sun_through.md)
- [Contrast and brightness grade after the tone map and the gamma, because on a linear HDR value a 0.5 pivot pushes a dark texel negative and ACES turns a negative warm color blue](reference_contrast_and_brightness_grade_after_the_tone_map.md)
- [The cluster light assignment binds the mesh lights buffer, so a lights buffer that grows must drop that bind group too, or every local light past the thirty-second reads a buffer nothing writes](reference_cluster_assign_bind_group_follows_the_lights_buffer.md)
- [A render pass reports its name from PassNode::name and is found by a separate lookup string, and a drift between them makes the pass draw nothing with no gate to catch it](reference_pass_name_must_match_lookup_constant.md)
- [Store ops are worked out per pass and resource, and a custom PostProcessPass must map scene_color](reference_per_pass_store_ops.md)
- [Only a few resources are safely shared between the mesh and skinned_mesh geometry passes, and the rest differ by format or layout](reference_mesh_skinned_passes_not_mergeable.md)
- [The outline mask and outline passes sit behind nightshade-render's debug_render cargo feature, which is not on by default](reference_outline_needs_debug_render_feature.md)
- [An integer carried in a texture coordinate is interpolated, so floor on it flips at whole numbers. Round it and sample the middle of the cell](reference_index_in_a_uv_needs_rounding.md)
- [wgpu's default 128MB max_storage_buffer_binding_size caps meshlet scenes at a few million unique vertices, and the meshlet feature raises it to the adapter's limit](reference_meshlet_storage_binding_limit.md)
- [Compute bind groups cap at 8 storage buffers on the WebGPU and wasm baseline, so pack or merge rather than adding a ninth](reference_compute_storage_buffer_limit.md)
- [The camera that renders last is the one the swapchain shows, so an offscreen camera goes before the viewer in required_cameras](reference_last_dispatched_camera_reaches_the_swapchain.md)
- [An oblique projection for a portal or a mirror needs its own frustum for the cull, or geometry in view is dropped](reference_oblique_projection_breaks_the_cull_frustum.md)
- [A pan-orbit camera's pitch is positive above the focus, so a negative pitch opens the app under the ground looking up](reference_pan_orbit_pitch_sign.md)
- [Difference two captures of a still scene to find what flickers, with the sponza app and ffmpeg](reference_flicker_diagnosis_capture_pair.md)
- [Three WGSL rules that naga and just lint-shaders accept and a browser refuses, so the fault only shows on a page load](reference_a_browser_rejects_what_the_gates_accept.md)
- [A Vulkan pass installed at PassStage::Scene has its draw called once in every color scope, so a pass that draws in one of them must gate on record.scope](reference_vulkan_scene_pass_draws_in_every_color_scope.md)

## The interface

- [The retained UI runs only where UiPlugin is composed, and RetainedUiRuntime.visible is the switch](reference_retained_ui_enable.md)
- [A lone space measures as zero width in the retained UI, and a world rectangle must be left unpainted by every layer above it](reference_retained_ui_space_and_hole.md)
- [Retained-UI button clicks are unreliable on touch, so hit-test each touch start against the node rect yourself](reference_retained_ui_touch_clicks_fragile.md)
- [UiLayoutRoot absolute_scale is consumed in three layout paths, so a root that scales in one and not the others leaves stragglers](reference_absolute_scale_three_paths.md)
- [apps/ui/gallery is the only caller of the retained UI widget constructors, so a dead-code sweep measures the gallery](reference_gallery_is_the_ui_coverage_gate.md)
- [The viewer tears its interface down and builds it again when the window crosses 700 logical pixels, because a boundary is fixed when the tree is built](reference_viewer_layout_rebuilds_on_width.md)
- [On wasm the canvas uses a capped device-pixel ratio but winit reports pointer coords at the raw ratio, and the two must be reconciled](reference_wasm_dpr_pointer_mismatch.md)

## Art

- [Mixamo glTF bones carry a mixamorig prefix and the engine stores the raw name, so resolve bones by suffix](reference_mixamo_bone_name_prefix.md)
- [Kenney racing kit tile geometry - 10x10 tiles, straight road along local Z, corner joins +Z to -X](reference_kenney_racing_tile_edges.md)
- [In the nightshade world an increasing yaw turns a (sin, cos) heading to the LEFT, so steering must subtract](reference_heading_yaw_turns_left.md)

## Building and shipping

- [In `just`, command-line `VAR=value` overrides must come BEFORE the recipe name, never after](reference_just_variable_overrides.md)
- [A second parallel build exhausts this machine and leaves artifacts that read as corruption, so rerun alone with -j 6](reference_parallel_cargo_builds_oom.md)
- [Main can move under you and backward, so fetch right before pushing and compare trees rather than hashes](reference_concurrent_sessions_rebase_main_upstream.md)
- [Merging crates silently kills feature gates that crossed the old boundary, and the build stays green while the behavior disappears](reference_cross_crate_feature_forwarding_dies_on_a_merge.md)
- [A web app's index.html must carry data-bin=<package>-run, because the package emits both a cdylib and a binary and trunk cannot choose on its own](reference_no_target_name_in_index_html.md)
- [An exported web build plays over HTTP and shows a blank page from a file:// address, so just serve-export exists to give it an address](reference_a_wasm_export_needs_a_server.md)
- [just lint-wasm runs clippy, which stops before the linker, so a native library in the graph passes every gate and breaks the web build](reference_lint_wasm_never_links.md)
- [An app that fetches over the web needs android.permission.INTERNET in its manifest.yaml, and template/manifest.yaml does not carry it](reference_android_manifest_needs_internet.md)
- [Each backend names every capability that draws, and the frame-type move was measured and refused](reference_backend_names_every_drawing_capability.md)
