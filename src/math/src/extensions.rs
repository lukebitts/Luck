use super::{Vector3, Vector4, Matrix4, normalize, cross, dot};
use num::traits::{Zero, One};

/// Returns a look at matrix from the supplied parameters. Eye is the camera position, center is
/// the location you want the camera to point, up is the up direction in whichever abstraction
/// you are working with (usually `Vector3::new(0, 1, 0)`).
pub fn look_at(eye: Vector3<f32>, center: Vector3<f32>, up: Vector3<f32>) -> Matrix4<f32> {
    let f = normalize(center - eye);
    let s = normalize(cross(up, f));
    let u = cross(f, s);

    let mut result = Matrix4::one();
    result.c0.x = s.x;
    result.c1.x = s.y;
    result.c2.x = s.z;
    result.c0.y = u.x;
    result.c1.y = u.y;
    result.c2.y = u.z;
    result.c0.z = f.x;
    result.c1.z = f.y;
    result.c2.z = f.z;
    result.c3.x = -dot(s, eye);
    result.c3.y = -dot(u, eye);
    result.c3.z = -dot(f, eye);
    result
}

/// Translates a matrix by a vector3.
pub fn translate(m: Matrix4<f32>, v: Vector3<f32>) -> Matrix4<f32> {
    let mut m = m;

    m.c3.x += v.x;
    m.c3.y += v.y;
    m.c3.z += v.z;

    m
}

/// Scales a matrix by a vector3.
pub fn scale(m: Matrix4<f32>, v: Vector3<f32>) -> Matrix4<f32> {
    let mut res = Matrix4::one();
    res.c0 = m[0] * v[0];
    res.c1 = m[1] * v[1];
    res.c2 = m[2] * v[2];
    res.c3 = m[3];
    res
}

/// Returns an orthogonal matrix from the camera parameters.
pub fn ortho(left: f32,
             right: f32,
             bottom: f32,
             top: f32,
             near_val: f32,
             far_val: f32)
             -> Matrix4<f32> {
    let mut res = Matrix4::one();
    res.c0.x = 2.0 / (right - left);
    res.c1.y = 2.0 / (top - bottom);
    res.c3.x = -(right + left) / (right - left);
    res.c3.y = -(top + bottom) / (top - bottom);

    res.c2.z = 1.0 / (far_val - near_val);
    res.c3.z = -near_val / (far_val - near_val);

    res
}

/// Returns a perspective matrix from the camera parameters.
pub fn frustum(left: f32,
               right: f32,
               bottom: f32,
               top: f32,
               near_val: f32,
               far_val: f32)
               -> Matrix4<f32> {
    let mut result = Matrix4::zero();

    result.c0.x = (2.0 * near_val) / (right - left);
    result.c1.y = (2.0 * near_val) / (top - bottom);
    result.c2.x = (right + left) / (right - left);
    result.c2.y = (top + bottom) / (top - bottom);
    result.c2.w = 1.0;

    result.c2.z = far_val / (far_val - near_val);
    result.c3.z = -(far_val * near_val) / (far_val - near_val);

    result
}

/// The result of a call to `is_box_in_frustum`
#[derive(Eq, PartialEq)]
pub enum FrustumTestResult {
    ///
    OUTSIDE = 0,
    ///
    INSIDE = 1,
    ///
    INTERSECT = 3,
}

fn vector_to_index(v: Vector3<f32>) -> i32 {
    let mut idx = 0;
    if v.z >= 0.0 {
        idx |= 1;
    }
    if v.y >= 0.0 {
        idx |= 2;
    }
    if v.x >= 0.0 {
        idx |= 4;
    }
    idx
}

fn half_plane_test(p: Vector3<f32>, normal: Vector3<f32>, offset: f32) -> i32 {
    let dist = dot(p, normal) + offset;
    if dist > 0.02 {
        1
    } else if dist < -0.02 {
        0
    } else {
        2
    }
}

/// Returns true if a box (or aabb) is inside the defined 6 plane frustrum.
pub fn is_box_in_frustum(origin: Vector3<f32>,
                         half_dim: Vector3<f32>,
                         planes: [Vector4<f32>; 6])
                         -> FrustumTestResult {
    let corner_offsets = [Vector3::new(-1.0, -1.0, -1.0),
                          Vector3::new(-1.0, -1.0, 1.0),
                          Vector3::new(-1.0, 1.0, -1.0),
                          Vector3::new(-1.0, 1.0, 1.0),
                          Vector3::new(1.0, -1.0, -1.0),
                          Vector3::new(1.0, -1.0, 1.0),
                          Vector3::new(1.0, 1.0, -1.0),
                          Vector3::new(1.0, 1.0, 1.0)];
    let mut ret = 1;
    for plane in &planes {
        let plane_normal = Vector3::new(plane.x, plane.y, plane.z);
        let mut idx = vector_to_index(plane_normal);

        let mut test_point = origin + half_dim * corner_offsets[idx as usize];

        if half_plane_test(test_point, plane_normal, plane.w) == 0 {
            ret = 0;
            break;
        }

        idx = vector_to_index(-plane_normal);
        test_point = origin + half_dim * corner_offsets[idx as usize];

        if half_plane_test(test_point, plane_normal, plane.w) == 0 {
            ret |= 2;
        }
    }

    match ret {
        0 => FrustumTestResult::OUTSIDE,
        1 => FrustumTestResult::INSIDE,
        3 => FrustumTestResult::INTERSECT,
        _ => unreachable!(),
    }
}

/// Normalizes x and y. Also makes sure y is orthogonal to x.
pub fn orthonormalize(x: &mut Vector3<f32>, y: &mut Vector3<f32>) {
    *x = normalize(*x);
    *y = normalize(*x - *y * dot(*y, *x));
}
