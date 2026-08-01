use crate::resources::TemplateResources;

pub fn query_pong_count(template_resources: &TemplateResources) -> u64 {
    template_resources.example.pong_count
}
