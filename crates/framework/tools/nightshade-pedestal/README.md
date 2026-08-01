# nightshade-pedestal

`nightshade-pedestal` is the scaffolding an app composes to stand one model under a sky on a pan orbit camera. A program says which sample model it wants, which sky bytes go behind it and where the camera stands, and the crate fetches the model, spawns it, frames the camera on its reach and loads the skybox. Nine rendering apps compose it, so each one is left holding only the setting its own feature is about.

## Composing it

```rust
app.insert_resource(Pedestal {
    model: theme::MODEL,
    sky: theme::SKY,
    back: theme::CAMERA_BACK,
    yaw: theme::CAMERA_YAW,
    pitch: theme::CAMERA_PITCH,
});
app.add_system(Stage::Startup, systems::setup::settle_the_look);
app.add_plugin(nightshade_pedestal::PedestalPlugin::titled(theme::TITLE));
```

`PedestalPlugin::titled` composes `ReadoutPlugin` with that title, inserts `PendingModel`, and registers `initialize` on `Stage::Startup` and `arrive` on `Stage::Update`. The program inserts `Pedestal` before the plugin runs, because the startup system reads it to place the camera and ask for the model.

## Using it

```rust
crate::commands::load_the_sky(
    resources.res_mut::<nightshade_graphics::commands::CommandQueues>(),
    &pedestal,
);
*resources.res_mut::<PendingModel>() = crate::commands::ask_for_the_model(
    resources.res_mut::<nightshade_gltf::Loading>(),
    &pedestal,
);
```

`Pedestal` names the sample model, the sky bytes, and the back, yaw and pitch the camera opens at, and it defaults to the Box model with no sky. `ask_for_the_model` turns the model name into a sample model URL and starts the fetch, returning the `PendingModel` that carries the load handle. `load_the_sky` queues the HDR skybox and returns at once when the sky is empty. The update system takes the load outcome when it arrives, spawns the first prefab of the import, and frames the pan orbit camera on the reach of the model scaled by `back`.

## Driving it

```
just example pedestal
```

A right drag orbits the camera around the model and the scroll wheel pulls in and out. The example stands one sample model under an HDR sky with bloom on, which is the scene every rendering app starts from.
