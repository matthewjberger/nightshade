mod backend;
mod data;
mod plugin;
mod queries;
mod resources;
mod systems;
mod theme;

use plugin::BoidsGpuPlugin;

pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = nightshade_app::compose::<resources::BoidsArguments>(
        nightshade_app::Opening::titled("Boids GPU").without_vsync(),
    );

    #[cfg(feature = "wgpu")]
    app.add_plugin(backend::wgpu::BoidsWgpuPlugin);

    #[cfg(feature = "vulkan")]
    app.add_plugin(backend::vulkan::BoidsVulkanPlugin);

    app.add_plugin(nightshade_camera_controller::CameraControllerPlugin)
        .add_plugin(nightshade_ui::UiPlugin::new())
        .add_plugin(BoidsGpuPlugin)
        .run()
}

nightshade_platform::android_main!(launch);
