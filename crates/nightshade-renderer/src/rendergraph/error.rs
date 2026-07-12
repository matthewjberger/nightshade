use super::ResourceId;

#[derive(Debug, thiserror::Error)]
pub enum RenderGraphError {
    #[error("Slot '{slot}' not found in pass '{pass}' resource mappings")]
    SlotNotFound { slot: String, pass: String },

    #[error("Resource '{resource}' (id: {id:?}) not bound")]
    ResourceNotBound { resource: String, id: ResourceId },

    #[error("Resource '{resource}' descriptor not found (id: {id:?})")]
    DescriptorNotFound { resource: String, id: ResourceId },

    #[error("Type mismatch: {operation} called on {actual_type} resource '{resource}'")]
    TypeMismatch {
        operation: String,
        actual_type: String,
        resource: String,
    },

    #[error("Pass '{pass}': slot '{slot}' not provided in mappings")]
    SlotNotMapped { pass: String, slot: String },

    #[error("Cannot resize external resource '{resource}'")]
    CannotResizeExternal { resource: String },

    #[error("Cannot resize buffer '{resource}' with width/height")]
    CannotResizeBuffer { resource: String },

    #[error("Cannot resize non-transient resource '{resource}'")]
    CannotResizeNonTransient { resource: String },

    #[error("Render graph contains a cycle (involves pass '{pass}')")]
    CyclicDependency { pass: String },

    #[error("Sub-graph '{sub_graph}' not found")]
    SubGraphNotFound { sub_graph: String },

    #[error("Sub-graph input '{input}' expects {expected} but received {received}")]
    SubGraphInputTypeMismatch {
        input: String,
        expected: String,
        received: String,
    },

    #[error("Resource '{resource}' (id: {id:?}) not found")]
    ResourceNotFound { resource: String, id: ResourceId },

    #[error("Pass '{pass}' not found")]
    PassNotFound { pass: String },
}

pub type Result<T> = std::result::Result<T, RenderGraphError>;
