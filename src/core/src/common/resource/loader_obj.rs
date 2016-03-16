//! A module for the OBJ loader.

use wavefront_obj;
use super::{Resource, MeshResource, ResourceLoader, ResourceLoadError, FileType};
use super::super::Vertex;
use std::rc::Rc;
use std;

impl std::convert::From<wavefront_obj::ParseError> for ResourceLoadError {
    fn from(e: wavefront_obj::ParseError) -> Self {
        // TODO: Use the fields inside the ParseError to build the error string
        ResourceLoadError::InvalidFile(format!("{:?}", e))
    }
}

/// A resource loader for OBJ files.
pub struct ObjResourceLoader;
impl ResourceLoader for ObjResourceLoader {
    /// Returns "obj"
    fn extensions(&self) -> Box<[&str]> {
        Box::new(["obj"])
    }

    /// Returns FileType::Text
    fn file_type(&self) -> FileType {
        FileType::Text
    }

    /// Parses an OBJ file from the `obj_data` string and returns a MeshResource as a Resource.
    fn load_from_text(&self, obj_data: &str) -> Result<Rc<Resource>, ResourceLoadError> {
        use wavefront_obj::obj::{Shape, parse};
        use std::collections::hash_map::{HashMap, Entry};

        let obj = try!(parse(obj_data.to_string()));
        let object = match obj.objects.first() {
            Some(o) => o,
            None => return Err(ResourceLoadError::InvalidFile("File has no objects.".to_string())),
        };

        let mut vb = Vec::new();
        let mut ib = Vec::new();
        let mut cache = HashMap::new();
        fn map(v1: (usize, Option<usize>, Option<usize>),
               index_vec: &mut Vec<u32>,
               vertex_vec: &mut Vec<Vertex>,
               object: &wavefront_obj::obj::Object,
               cache: &mut HashMap<(usize, usize, usize), u32>)
               -> Result<(), ResourceLoadError> {
            match v1 {
                (vi, Some(ti), Some(ni)) => {
                    let index = match cache.entry((vi, ti, ni)) {
                        Entry::Vacant(entry) => {
                            let v = object.vertices[vi];
                            let t = object.tex_vertices[ti];
                            let n = object.normals[ni];
                            let vertex = Vertex {
                                position: [v.x as f32, v.y as f32, v.z as f32],
                                normal: [n.x as f32, n.y as f32, n.z as f32],
                                tex_coord: [t.x as f32, t.y as f32],
                                tangent: [0.0, 0.0, 0.0, 0.0],
                            };
                            let index = vertex_vec.len() as u32;
                            vertex_vec.push(vertex);
                            entry.insert(index);
                            index
                        }
                        Entry::Occupied(entry) => *entry.get(),
                    };
                    index_vec.push(index);
                }
                _ => {
                    return Err(ResourceLoadError::InvalidFile("Object is missing normals or \
                                                               texture coordinates."
                                                                  .to_string()))
                }
            }
            Ok(())
        };
        for geometry in &object.geometry {
            for shape in &geometry.shapes {
                match *shape {
                    Shape::Triangle(v1, v2, v3) => {
                        try!(map(v1, &mut ib, &mut vb, &object, &mut cache));
                        try!(map(v2, &mut ib, &mut vb, &object, &mut cache));
                        try!(map(v3, &mut ib, &mut vb, &object, &mut cache));
                    }
                    // This is unreachable since wavefront_obj automatically triangulates meshs.
                    _ => {
                        return Err(ResourceLoadError::InvalidFile("Object has shapes other than \
                                                                   triangles"
                                                                      .to_string()))
                    }
                }
            }
        }
        let mut m = MeshResource {
            vertices: vb,
            indices: ib,
        };
        calculate_mesh_tangents(&mut m);
        Ok(Rc::new(m))
    }
}

fn calculate_mesh_tangents(mesh: &mut MeshResource) {
    use luck_math;
    use num::traits::Zero;
    use luck_math::{Vector3, orthonormalize, cross, dot};

    let indices = mesh.indices.clone();
    let vertices = mesh.vertices.clone();

    let index_count = indices.len();
    let vertex_count = vertices.len();

    let mut tan1: Vec<Vector3<f32>> = vec![Vector3::zero(); vertex_count];
    let mut tan2: Vec<Vector3<f32>> = vec![Vector3::zero(); vertex_count];

    // When step_by is stable, this line will be (0..index_count).step_by(3)
    for a in (0..index_count / 3).map(|i| i * 3) {
        let i1 = indices[a];
        let i2 = indices[a + 1];
        let i3 = indices[a + 2];

        let v1 = vertices[i1 as usize];
        let v2 = vertices[i2 as usize];
        let v3 = vertices[i3 as usize];

        let w1 = v1.tex_coord;
        let w2 = v2.tex_coord;
        let w3 = v3.tex_coord;

        let x1 = v2.position[0] - v1.position[0];
        let x2 = v3.position[0] - v1.position[0];
        let y1 = v2.position[1] - v1.position[1];
        let y2 = v3.position[1] - v1.position[1];
        let z1 = v2.position[2] - v1.position[2];
        let z2 = v3.position[2] - v1.position[2];

        let s1 = w2[0] - w1[0];
        let s2 = w3[0] - w1[0];
        let t1 = w2[1] - w1[1];
        let t2 = w3[1] - w1[1];

        let r = 1.0 / (s1 * t2 - s2 * t1);

        let sdir = Vector3::new((t2 * x1 - t1 * x2) * r,
                                (t2 * y1 - t1 * y2) * r,
                                (t2 * z1 - t1 * z2) * r);
        let tdir = Vector3::new((s1 * x2 - s2 * x1) * r,
                                (s1 * y2 - s2 * y1) * r,
                                (s1 * z2 - s2 * z1) * r);

        tan1[i1 as usize] = tan1[i1 as usize] + sdir;
        tan1[i2 as usize] = tan1[i2 as usize] + sdir;
        tan1[i3 as usize] = tan1[i3 as usize] + sdir;

        tan2[i1 as usize] = tan2[i1 as usize] + tdir;
        tan2[i2 as usize] = tan2[i2 as usize] + tdir;
        tan2[i3 as usize] = tan2[i3 as usize] + tdir;
    }

    for a in 0..vertex_count {
        let n = vertices[a].normal;
        let t = tan1[a];

        let mut vn = Vector3::new(n[0], n[1], n[2]);
        let mut vt = Vector3::new(t[0], t[1], t[2]);
        luck_math::orthonormalize(&mut vn, &mut vt);
        mesh.vertices[a].normal = [vn.x, vn.y, vn.z];

        mesh.vertices[a].tangent[0] = vt.x;
        mesh.vertices[a].tangent[1] = vt.y;
        mesh.vertices[a].tangent[2] = vt.z;

        mesh.vertices[a].tangent[3] = if dot(cross(vn, vt), tan2[a]) < 0.0 {
            -1.0
        } else {
            1.0
        };
    }
}

#[cfg(test)]
mod test {
    use super::ObjResourceLoader;
    use super::super::{MeshResource, ResourceLoader, ResourceLoadError};
    use super::super::super::Vertex;

    #[test]
    fn mesh_creation() {

        let loader = ObjResourceLoader;

        match loader.load_from_text("a") {
            Err(ResourceLoadError::InvalidFile(_)) => (),
            _ => panic!(),
        }

        match loader.load_from_text("") {
            Err(ResourceLoadError::InvalidFile(_)) => (),
            _ => panic!(),
        }

        // Mesh without normals and tex coords (Also has no triangles, which is an error too, but
        // that is not the first error returned)
        let face = r"
        # Blender v2.71 (sub 0) OBJ File: ''
        # www.blender.org
        o Plane
        v -1.000000 0.000000 1.000000
        v 1.000000 0.000000 1.000000
        v -1.000000 0.000000 -1.000000
        v 1.000000 0.000000 -1.000000
        s off
        f 1 2 4 3";

        match loader.load_from_text(face) {
            Err(ResourceLoadError::InvalidFile(_)) => (),
            _ => panic!(),
        };

        let face = r"# Blender v2.71 (sub 0) OBJ File: ''
        # www.blender.org
        o Plane
        v -1.000000 0.000000 1.000000
        v 1.000000 0.000000 1.000000
        v -1.000000 0.000000 -1.000000
        v 1.000000 0.000000 -1.000000
        vt 0.000100 0.000100
        vt 0.999900 0.000100
        vt 0.999900 0.999900
        vt 0.000100 0.999900
        vn 0.000000 1.000000 0.000000
        s off
        f 1/1/1 2/2/1 4/3/1 3/4/1";

        let m = loader.load_from_text(face).unwrap().clone();
        let m = m.downcast_ref::<MeshResource>().unwrap();
        let v = vec![Vertex {
                         position: [-1.0, 0.0, 1.0],
                         normal: [0.0, 1.0, 0.0],
                         tex_coord: [0.0001, 0.0001],
                         tangent: [1.0, 0.0, 0.0, 1.0],
                     },
                     Vertex {
                         position: [1.0, 0.0, 1.0],
                         normal: [0.0, 1.0, 0.0],
                         tex_coord: [0.9999, 0.0001],
                         tangent: [1.0, 0.0, 0.0, 1.0],
                     },
                     Vertex {
                         position: [1.0, 0.0, -1.0],
                         normal: [0.0, 1.0, 0.0],
                         tex_coord: [0.9999, 0.9999],
                         tangent: [1.0, 0.0, 0.0, 1.0],
                     },
                     Vertex {
                         position: [-1.0, 0.0, -1.0],
                         normal: [0.0, 1.0, 0.0],
                         tex_coord: [0.0001, 0.9999],
                         tangent: [1.0, 0.0, 0.0, 1.0],
                     }];

        for vertex in v {
            assert!(m.vertices.contains(&vertex));
        }

        // TODO: Test indices too
    }

}
