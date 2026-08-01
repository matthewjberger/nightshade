use nightshade_ui::commands::builder::UiTreeBuilder;

pub fn type_scale(tree: &UiTreeBuilder) -> f32 {
    tree.theme_state().type_scale_target.max(0.1)
}
