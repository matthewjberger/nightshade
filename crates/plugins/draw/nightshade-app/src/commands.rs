use crate::resources::Opening;
use nightshade::app::App;
use nightshade::plugin::PluginGroup;

pub fn compose<A>(opening: Opening) -> App
where
    A: nightshade_cli::clap::Args + Default + Send + Sync + 'static,
{
    let mut app = App::new();

    #[cfg(feature = "vulkan")]
    app.add_plugins(
        nightshade_vulkan::DefaultPlugins::<A>::new()
            .set(nightshade_cli::CliPlugin::<A> {
                capture_frame: opening.capture_frame,
                ..Default::default()
            })
            .set(nightshade_log::LogPlugin {
                title: Some(opening.title.to_string()),
                ..Default::default()
            })
            .set(nightshade_window::WindowPlugin {
                title: opening.title.to_string(),
                size: opening.size,
            })
            .set(nightshade_vulkan::RenderPlugin {
                vsync: opening.vsync,
                frame_rate_limit: opening.frame_rate_limit,
            }),
    );

    #[cfg(feature = "wgpu")]
    app.add_plugins(
        nightshade_wgpu::DefaultPlugins::<A>::new()
            .set(nightshade_cli::CliPlugin::<A> {
                capture_frame: opening.capture_frame,
                ..Default::default()
            })
            .set(nightshade_log::LogPlugin {
                title: Some(opening.title.to_string()),
                ..Default::default()
            })
            .set(nightshade_window::WindowPlugin {
                title: opening.title.to_string(),
                size: opening.size,
            })
            .set(nightshade_wgpu::RenderPlugin {
                vsync: opening.vsync,
                frame_rate_limit: opening.frame_rate_limit,
            }),
    );

    #[cfg(feature = "wgpu")]
    app.add_plugin(nightshade_wgpu_shading::ShadingPlugin);
    #[cfg(feature = "wgpu")]
    app.add_plugin(nightshade_wgpu_shadow::ShadowRenderPlugin);
    #[cfg(feature = "wgpu")]
    app.add_plugin(nightshade_wgpu_sky::SkyRenderPlugin);
    #[cfg(feature = "wgpu")]
    app.add_plugin(nightshade_wgpu_geometry::GeometryRenderPlugin);
    #[cfg(feature = "wgpu")]
    app.add_plugin(nightshade_wgpu_outline::OutlineRenderPlugin);
    #[cfg(feature = "wgpu")]
    app.add_plugin(nightshade_wgpu_debug::DebugRenderPlugin);
    #[cfg(feature = "wgpu")]
    app.add_plugin(nightshade_wgpu_text::TextRenderPlugin);
    #[cfg(feature = "wgpu")]
    app.add_plugin(nightshade_wgpu_sprites::SpriteRenderPlugin);
    #[cfg(feature = "wgpu")]
    app.add_plugin(nightshade_wgpu_screenspace::ScreenSpaceRenderPlugin);
    #[cfg(feature = "wgpu")]
    app.add_plugin(nightshade_wgpu_paint::PaintRenderPlugin);
    #[cfg(all(feature = "wgpu", not(target_arch = "wasm32")))]
    app.add_plugin(nightshade_wgpu_acoustics::AcousticsRenderPlugin);
    #[cfg(feature = "wgpu")]
    app.add_plugin(nightshade_wgpu_near::NearRenderPlugin);
    #[cfg(feature = "wgpu")]
    app.add_plugin(nightshade_wgpu_sdf::SdfRenderPlugin);
    #[cfg(feature = "wgpu")]
    app.add_plugin(nightshade_wgpu_portals::PortalRenderPlugin);
    #[cfg(feature = "wgpu")]
    app.add_plugin(nightshade_wgpu_decals::DecalRenderPlugin);
    #[cfg(feature = "wgpu")]
    app.add_plugin(nightshade_wgpu_particles::ParticleRenderPlugin);

    app.add_plugin(crate::plugin::OpeningPlugin { opening });

    app
}
