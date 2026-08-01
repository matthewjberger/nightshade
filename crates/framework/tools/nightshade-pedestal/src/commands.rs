use crate::resources::{Pedestal, PendingModel};

pub fn ask_for_the_model(
    loading: &mut nightshade_gltf::Loading,
    pedestal: &Pedestal,
) -> PendingModel {
    let url = if pedestal.compressed {
        nightshade_gltf::queries::sample_model_url_compressed(pedestal.model)
    } else {
        nightshade_gltf::queries::sample_model_url(pedestal.model)
    };
    PendingModel {
        handle: Some(nightshade_load::load(
            loading,
            nightshade_load::Source::Url(url),
        )),
    }
}

pub fn load_the_sky(
    queues: &mut nightshade_graphics::resources::queues::CommandQueues,
    pedestal: &Pedestal,
) {
    if pedestal.sky.is_empty() {
        return;
    }
    nightshade_graphics::commands::queues::load_hdr_skybox(queues, pedestal.sky.to_vec());
}
