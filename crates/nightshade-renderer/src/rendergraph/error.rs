use super::ResourceId;

/// A failure raised while building, resolving, or resizing the render graph.
#[derive(Debug, thiserror::Error)]
pub enum RenderGraphError {
    /// A pass looked up a slot that its resource mappings do not define.
    #[error("Slot '{slot}' not found in pass '{pass}' resource mappings")]
    SlotNotFound {
        /// The slot name the pass looked up.
        slot: String,
        /// The pass that made the lookup.
        pass: String,
    },

    /// A resource id resolved to no bound GPU resource.
    #[error("Resource '{resource}' (id: {id:?}) not bound")]
    ResourceNotBound {
        /// The resource name.
        resource: String,
        /// The unresolved resource id.
        id: ResourceId,
    },

    /// A resource id has no registered descriptor.
    #[error("Resource '{resource}' descriptor not found (id: {id:?})")]
    DescriptorNotFound {
        /// The resource name.
        resource: String,
        /// The id whose descriptor is missing.
        id: ResourceId,
    },

    /// An operation ran against a resource of the wrong kind, such as a buffer
    /// operation on a texture.
    #[error("Type mismatch: {operation} called on {actual_type} resource '{resource}'")]
    TypeMismatch {
        /// The operation that was attempted.
        operation: String,
        /// The resource's actual kind.
        actual_type: String,
        /// The resource name.
        resource: String,
    },

    /// A pass slot was left unmapped when the graph was assembled.
    #[error("Pass '{pass}': slot '{slot}' not provided in mappings")]
    SlotNotMapped {
        /// The pass with the unmapped slot.
        pass: String,
        /// The slot left unmapped.
        slot: String,
    },

    /// A resize targeted an externally owned resource the graph cannot resize.
    #[error("Cannot resize external resource '{resource}'")]
    CannotResizeExternal {
        /// The external resource that cannot be resized.
        resource: String,
    },

    /// A width/height resize targeted a buffer, which has no dimensions.
    #[error("Cannot resize buffer '{resource}' with width/height")]
    CannotResizeBuffer {
        /// The buffer resource that has no dimensions.
        resource: String,
    },

    /// A resize targeted a resource that is not transient.
    #[error("Cannot resize non-transient resource '{resource}'")]
    CannotResizeNonTransient {
        /// The non-transient resource that cannot be resized.
        resource: String,
    },

    /// Pass dependencies form a cycle, so no valid execution order exists.
    #[error("Render graph contains a cycle (involves pass '{pass}')")]
    CyclicDependency {
        /// A pass that participates in the cycle.
        pass: String,
    },

    /// A referenced sub-graph is not registered.
    #[error("Sub-graph '{sub_graph}' not found")]
    SubGraphNotFound {
        /// The sub-graph name that is not registered.
        sub_graph: String,
    },

    /// A sub-graph input was given a resource of the wrong type.
    #[error("Sub-graph input '{input}' expects {expected} but received {received}")]
    SubGraphInputTypeMismatch {
        /// The sub-graph input slot name.
        input: String,
        /// The resource type the input expects.
        expected: String,
        /// The resource type that was supplied.
        received: String,
    },

    /// A resource id resolved to no registered resource.
    #[error("Resource '{resource}' (id: {id:?}) not found")]
    ResourceNotFound {
        /// The resource name.
        resource: String,
        /// The unregistered resource id.
        id: ResourceId,
    },

    /// A referenced pass is not registered in the graph.
    #[error("Pass '{pass}' not found")]
    PassNotFound {
        /// The pass name that is not registered.
        pass: String,
    },
}

/// Result type for render graph operations, defaulting the error to
/// [`RenderGraphError`].
pub type Result<T> = std::result::Result<T, RenderGraphError>;
