#![allow(missing_docs)]

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coord: [f32; 2],
    pub tangent: [f32; 4],
}

implement_vertex!(Vertex, position, normal, tex_coord, tangent);
