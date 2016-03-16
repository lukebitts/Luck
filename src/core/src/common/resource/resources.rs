use mopa;
use super::super::Vertex;

/// A trait that every resource type has to implement, so we can downcast to the correct type.
pub trait Resource : mopa::Any {}
mopafy!(Resource);

/// A mesh resource. Composed of a list of vertices and a list of indices.
#[derive(Clone)]
pub struct MeshResource {
    /// A list of the mesh's vertices.
    pub vertices: Vec<Vertex>,
    /// A list of the mesh's indices.
    pub indices: Vec<u32>,
}

impl Resource for MeshResource {}
