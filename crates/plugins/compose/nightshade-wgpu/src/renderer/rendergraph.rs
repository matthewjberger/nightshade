mod error;
mod graph;
mod pass;
mod resources;

pub use error::{RenderGraphError, Result};
pub use graph::{
    RenderGraph, render_graph_add_color_texture, render_graph_add_depth_texture,
    render_graph_add_pass, render_graph_add_pass_at, render_graph_bind_pass_slot,
    render_graph_compile, render_graph_enable_only_passes, render_graph_execute,
    render_graph_execute_with_phase, render_graph_get_pass_mut, render_graph_get_texture,
    render_graph_get_texture_view, render_graph_material_update, render_graph_new,
    render_graph_pass_names_in_stage, render_graph_resize_all_transient_proportional,
    render_graph_resize_passes, render_graph_resize_transient_resource,
    render_graph_restore_pass_enabled_states, render_graph_save_pass_enabled_states,
    render_graph_set_external_texture, render_graph_set_pass_enabled,
};
pub use pass::{
    ExecutePhase, MaterialUpdate, PassDecl, PassExecutionContext, PassNode, PassPosition,
    PassStage, ResourcePublisher, ResourceReader,
};
pub use resources::ResourceId;
