use crate::systems::bilateral_blur::BilateralBlurPass;
use crate::systems::ssao::SsaoPass;
use crate::systems::ssgi::SsgiPass;
use crate::systems::ssr::SsrPass;
use nightshade_wgpu::backend::{PassBuild, RenderGraph, RenderInputs, install_scene_pass};
use nightshade_wgpu::backend::{PassNode, ResourceId};
use nightshade_wgpu::wgpu;

type Install = (
    Box<dyn PassNode<RenderInputs>>,
    Vec<(&'static str, ResourceId)>,
);

pub fn install_screen_space_passes(
    graph: &mut RenderGraph<RenderInputs>,
    build: PassBuild,
    _: wgpu::TextureFormat,
) {
    let depth = build.targets.depth;
    let view_normals = build.targets.view_normals;
    let scene_color = build.targets.scene_color;
    let ssao_raw = build.targets.ssao_raw;
    let ssao = build.targets.ssao;
    let ssgi_raw = build.targets.ssgi_raw;
    let ssgi = build.targets.ssgi;
    let ssr_raw = build.targets.ssr_raw;
    let ssr = build.targets.ssr;

    let installs: [Install; 6] = [
        (
            Box::new(SsaoPass::new(build)),
            vec![
                ("depth", depth),
                ("view_normals", view_normals),
                ("ssao_raw", ssao_raw),
            ],
        ),
        (
            Box::new(
                BilateralBlurPass::ssao(build)
                    .when(|configs| configs.scene.active_view.ssao_enabled),
            ),
            vec![
                ("ssao_raw", ssao_raw),
                ("depth", depth),
                ("view_normals", view_normals),
                ("ssao", ssao),
            ],
        ),
        (
            Box::new(SsgiPass::new(build)),
            vec![
                ("depth", depth),
                ("view_normals", view_normals),
                ("scene_color", scene_color),
                ("ssgi_raw", ssgi_raw),
            ],
        ),
        (
            Box::new(BilateralBlurPass::ssgi(build)),
            vec![
                ("ssgi_raw", ssgi_raw),
                ("depth", depth),
                ("view_normals", view_normals),
                ("ssgi", ssgi),
            ],
        ),
        (
            Box::new(SsrPass::new(build)),
            vec![
                ("depth", depth),
                ("view_normals", view_normals),
                ("scene_color", scene_color),
                ("ssr_raw", ssr_raw),
            ],
        ),
        (
            Box::new(BilateralBlurPass::ssr(build)),
            vec![
                ("ssr_raw", ssr_raw),
                ("depth", depth),
                ("view_normals", view_normals),
                ("ssr", ssr),
            ],
        ),
    ];

    for (pass, slots) in installs {
        if let Err(error) = install_scene_pass(graph, pass, &slots) {
            panic!("a screen space pass could not be installed: {error}");
        }
    }
}
