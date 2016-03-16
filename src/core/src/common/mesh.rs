use glium;
use std;
use luck_math;
use luck_math::{Vector3, Aabb};
use glium::backend::glutin_backend::GlutinFacade;
use super::resource::MeshResource;

use super::Vertex;

// TODO: There are some things wrong with this type:
// * There's no way to change the vertices in a mesh and update them in the video card.
// * There's no way to release the memory in the cpu and keep the mesh in the gpu.

/// A description of the possible errors you get when creating a mesh.
#[derive(Debug)]
pub enum MeshCreationError {
    /// Returned when glium can't create the vertex buffer.
    VertexBufferCreationError(glium::vertex::BufferCreationError),
    /// Returned when glium can't create the index buffer.
    IndexBufferCreationError(glium::index::BufferCreationError),
}

impl std::convert::From<glium::vertex::BufferCreationError> for MeshCreationError {
    fn from(e: glium::vertex::BufferCreationError) -> Self {
        MeshCreationError::VertexBufferCreationError(e)
    }
}

impl std::convert::From<glium::index::BufferCreationError> for MeshCreationError {
    fn from(e: glium::index::BufferCreationError) -> Self {
        MeshCreationError::IndexBufferCreationError(e)
    }
}

/// A struct that holds the definition of a mesh in memory and in the GPU.
pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
    vertex_buffer: glium::VertexBuffer<Vertex>,
    index_buffer: glium::IndexBuffer<u32>,
}

impl Mesh {
    /// Creates a new mesh from a MeshResource.
    pub fn from_resource(display: &GlutinFacade,
                         resource: &MeshResource)
                         -> Result<Self, MeshCreationError> {
        Ok(Mesh {
            vertices: resource.vertices.clone(),
            indices: resource.indices.clone(),
            vertex_buffer: try!(glium::VertexBuffer::new(display, &resource.vertices)),
            index_buffer: try!(glium::IndexBuffer::new(display,
                                                       glium::index::PrimitiveType::TrianglesList,
                                                       &resource.indices)),
        })
    }

    /// Returns a cube mesh.
    // # Panics
    // This function loads a static obj file at compile time. This file is guaranteed to be
    // valid so we ignore errors which could result in a panic. But it is unlikely.
    pub fn cube(display: &GlutinFacade) -> Self {
        use super::resource::loader_obj::ObjResourceLoader;
        use super::resource::ResourceLoader;
        let loader = ObjResourceLoader;

        let mesh_resource = loader.load_from_text(include_str!("../../static_assets/mesh/cube.\
                                                                obj"))
                                  .expect("");
        Mesh::from_resource(display, mesh_resource.downcast_ref::<MeshResource>().unwrap()).unwrap()

    }

    /// Returns a sphere mesh.
    // # Panics
    // This function loads a static obj file at compile time. This file is guaranteed to be
    // valid so we ignore errors which could result in a panic. But it is unlikely.
    pub fn sphere(display: &GlutinFacade) -> Self {
        use super::resource::loader_obj::ObjResourceLoader;
        use super::resource::ResourceLoader;
        let loader = ObjResourceLoader;

        let mesh_resource = loader.load_from_text(include_str!("../../static_assets/mesh/sphere.\
                                                                obj"))
                                  .expect("");
        Mesh::from_resource(display, mesh_resource.downcast_ref::<MeshResource>().unwrap()).unwrap()
    }

    /// Calculates the AABB of the mesh. If the mesh has no vertices, a default AABB is returned.
    pub fn calculate_aabb(&self) -> Aabb {
        if self.vertices.is_empty() {
            Aabb::default()
        } else {
            let mut min = Vector3::new(self.vertices[0].position[0],
                                       self.vertices[0].position[1],
                                       self.vertices[0].position[2]);
            let mut max = min.clone();

            for v in &self.vertices {
                min = luck_math::min(min,
                                     Vector3::new(v.position[0], v.position[1], v.position[2]));
                max = luck_math::max(max,
                                     Vector3::new(v.position[0], v.position[1], v.position[2]));
            }

            Aabb::new(min, max)
        }
    }
}

#[cfg(test)]
mod test {
    use glium;
    use glium::{DisplayBuild};
    use luck_math::{Aabb, Vector3};
    use super::Mesh;

    #[test]
    fn mesh_test() {
        let display = glium::glutin::WindowBuilder::new()
            .build_glium()
            .unwrap();

        let cube = Mesh::cube(&display);
        let sphere = Mesh::sphere(&display);

        assert_eq!(cube.calculate_aabb(), Aabb::new(Vector3::new(-0.5, -0.5, -0.5), Vector3::new(0.5, 0.5, 0.5)));
        assert_eq!(sphere.calculate_aabb(), Aabb::new(Vector3::new(-0.5, -0.5, -0.5), Vector3::new(0.5, 0.5, 0.5)));
    }
}
