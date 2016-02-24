//! A module for the AABB type. It also exposes an enum type for intersection tests.

use super::Vector3;
use num::traits::{Zero, One};

/// The return type of `Aabb::intersect`.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum IntersectionType {
    ///
    INSIDE,
    ///
    INTERSECT,
    ///
    OUTSIDE,
}

/// An AABB represented by two `Vector3`.
#[derive(Debug, Copy, Clone)]
pub struct Aabb {
    /// The minimum value.
    pub min: Vector3<f32>,
    /// The maximum value.
    pub max: Vector3<f32>,
}

impl Default for Aabb {
    /// Returns a null AABB.
    fn default() -> Aabb {
        let mut r = Aabb {
            min: Vector3::zero() + 1.0,
            max: Vector3::zero() - 1.0,
        };
        r.set_null();
        r
    }
}

impl Aabb {
    /// Returns a new instance of Aabb with the specified values.
    pub fn new(p1: Vector3<f32>, p2: Vector3<f32>) -> Self {
        let mut r = Aabb::default();
        r.extend_by_vec(p1);
        r.extend_by_vec(p2);
        r
    }

    /// Returns a new instance of Aabb considering a center and a radius.
    pub fn with_center(center: Vector3<f32>, radius: f32) -> Self {
        let mut r = Aabb::default();
        r.extend_by_radius(center, radius);
        r
    }

    /// Sets the Aabb to an invalid state.
    pub fn set_null(&mut self) {
        *self = Aabb {
            min: Vector3::new(1.0, 1.0, 1.0),
            max: Vector3::new(-1.0, -1.0, -1.0),
        };
    }

    /// Checks the Aabb state.
    pub fn is_null(&self) -> bool {
        self.min.x > self.max.x || self.min.y > self.max.y || self.min.z > self.max.z
    }

    /// Extends the Aabb size by a value. This value is subtracted from the min vector and
    /// added to the max vector.
    pub fn extend_by_value(&mut self, val: f32) {
        if !self.is_null() {
            self.min = self.min - Vector3::new(val, val, val);
            self.max = self.max + Vector3::new(val, val, val);
        }
    }

    /// Extends the Aabb by a Vector3. The min vector becomes a combination of itself and the
    /// parameter p, having the smallest value of both. The same goes for the max vector.
    pub fn extend_by_vec(&mut self, p: Vector3<f32>) {
        if !self.is_null() {
            self.min = super::min(p, self.min);
            self.max = super::max(p, self.max);
        } else {
            self.min = p;
            self.max = p;
        }
    }

    /// Extends an aabb by the specified radius.
    pub fn extend_by_radius(&mut self, p: Vector3<f32>, radius: f32) {
        let r = Vector3::<f32>::new(radius, radius, radius);
        if !self.is_null() {
            self.min = super::min(p - r, self.min);
            self.max = super::max(p - r, self.max);
        } else {
            self.min = p - r;
            self.max = p + r;
        }
    }

    /// Extends this aabb so the parameter aabb can fit within it.
    pub fn extend_by_aabb(&mut self, aabb: Aabb) {
        if !aabb.is_null() {
            self.extend_by_vec(aabb.min);
            self.extend_by_vec(aabb.max);
        }
    }

    /// TODO
    pub fn extend_disk(&mut self, c: Vector3<f32>, n: Vector3<f32>, r: f32) {
        if super::length(n) < 1e-12 {
            self.extend_by_vec(c);
        } else {
            let norm = super::normalize(n);
            let x = (1.0 - norm.x).sqrt() * r;
            let y = (1.0 - norm.y).sqrt() * r;
            let z = (1.0 - norm.y).sqrt() * r;
            self.extend_by_vec(c + Vector3::new(x, y, z));
            self.extend_by_vec(c - Vector3::new(x, y, z));
        }
    }

    /// Returns the diagonal of the AABB which is defined as Aabb::max - Aabb::min. If the Aabb
    /// is null, Vector3::zero() is returned.
    pub fn diagonal(&self) -> Vector3<f32> {
        if !self.is_null() {
            self.max - self.min
        } else {
            Vector3::zero()
        }
    }

    /// Returns the longest edge.
    pub fn longest_edge(&self) -> f32 {
        self.diagonal().x.max(self.diagonal().y).max(self.diagonal().z)
    }

    /// Returns the shortest edge.
    pub fn shortest_edge(&self) -> f32 {
        self.diagonal().x.min(self.diagonal().y).min(self.diagonal().z)
    }

    /// Returns the center of the aabb. If the Aabb is null, Vector3::zero is returned.
    pub fn center(&self) -> Vector3<f32> {
        if !self.is_null() {
            let d = self.diagonal();
            self.min + (d * 0.5)
        } else {
            Vector3::zero()
        }
    }

    /// Translates the Aabb by a vector.
    pub fn translate(&mut self, v: Vector3<f32>) {
        if !self.is_null() {
            self.min = self.min + v;
            self.max = self.max + v;
        }
    }

    /// Scales the Aabb by s considering o as the origin.
    pub fn scale(&mut self, s: Vector3<f32>, o: Vector3<f32>) {
        if !self.is_null() {
            self.min = self.min - o;
            self.max = self.max - o;

            self.min = self.min * s;
            self.max = self.max * s;

            self.min = self.min + o;
            self.max = self.max + o;
        }
    }

    /// Returns true if both Aabb's overlap.
    pub fn overlaps(&self, bb: Aabb) -> bool {
        !((self.is_null() || bb.is_null()) || (bb.min.x > self.max.x || bb.max.x < self.min.x) ||
          (bb.min.y > self.max.y || bb.max.y < self.min.y) ||
          (bb.min.z > self.max.z || bb.max.z < self.min.z))
    }

    /// Calculates the intersection type between two Aabb's.
    pub fn intersect(&self, b: Aabb) -> IntersectionType {
        if self.is_null() || b.is_null() {
            return IntersectionType::OUTSIDE;
        }

        if (self.max.x < b.min.x) || (self.min.x > b.max.x) || (self.max.y < b.min.y) ||
           (self.min.y > b.max.y) || (self.max.z < b.min.z) || (self.min.z > b.max.z) {
            return IntersectionType::OUTSIDE;
        }

        if (self.min.x <= b.min.x) && (self.max.x >= b.max.x) && (self.min.y <= b.min.y) &&
           (self.max.y >= b.max.y) && (self.min.z <= b.min.z) &&
           (self.max.z >= b.max.z) {
            return IntersectionType::INSIDE;
        }

        IntersectionType::INTERSECT
    }

    /// Returns true if both Aabb's a sufficiently similar.
    pub fn similar_to(&self, b: Aabb, diff: f32) -> bool {
        if self.is_null() || b.is_null() {
            return false;
        }

        let acceptable_diff = ((self.diagonal() + b.diagonal()) / 2.0) * diff;
        let mut min_diff = self.min - b.min;
        min_diff = Vector3::new(min_diff.x.abs(), min_diff.y.abs(), min_diff.z.abs());
        if min_diff.x > acceptable_diff.x {
            return false;
        }
        if min_diff.y > acceptable_diff.y {
            return false;
        }
        if min_diff.z > acceptable_diff.z {
            return false;
        }

        let mut max_diff = self.max - b.max;
        max_diff = Vector3::new(max_diff.x.abs(), max_diff.y.abs(), max_diff.z.abs());
        if max_diff.x > acceptable_diff.x {
            return false;
        }
        if max_diff.y > acceptable_diff.y {
            return false;
        }
        if max_diff.z > acceptable_diff.z {
            return false;
        }

        true
    }

    /// Returns the perimeter of the Aabb.
    pub fn perimeter(&self) -> f32 {
        let wx = self.max.x - self.min.x;
        let wy = self.max.y - self.min.y;

        2.0 * (wx + wy)
    }

    /// Combines two Aabb's and the result in as Aabb that encompasses both parameters.
    pub fn combine(&mut self, aabb1: Aabb, aabb2: Aabb) {
        self.min = Vector3::new(super::min(aabb1.min.x, aabb2.min.x),
                                super::min(aabb1.min.y, aabb2.min.y),
                                super::min(aabb1.min.z, aabb2.min.z));
        self.max = Vector3::new(super::max(aabb1.max.x, aabb2.max.x),
                                super::max(aabb1.max.y, aabb2.max.y),
                                super::max(aabb1.max.z, aabb2.max.z));
    }

    /// Returns true if this Aabb contains the aabb parameter.
    pub fn contains(&self, aabb: Aabb) -> bool {
        let mut result = true;
        result = result && self.min.x <= aabb.min.x;
        result = result && self.min.y <= aabb.min.y;
        result = result && aabb.max.x <= self.max.x;
        result = result && aabb.max.y <= self.max.y;
        result
    }

    /// Returns a vertex list for this Aabb. Useful for debug rendering or operations that
    /// require every point and not just the min and max.
    pub fn vertices(&self) -> [Vector3<f32>; 8] {
        [self.min,
         Vector3::new(self.max.x, self.min.y, self.min.z),
         Vector3::new(self.min.x, self.max.y, self.min.z),
         Vector3::new(self.min.x, self.min.y, self.max.z),
         Vector3::new(self.min.x, self.max.y, self.max.z),
         Vector3::new(self.max.x, self.min.y, self.max.z),
         Vector3::new(self.max.x, self.max.y, self.min.z),
         self.max]
    }

    /// This function considers the Aabb as a box, rotates it and then calculates a new Aabb for
    /// the rotated box. Rotating the same Aabb over and over will only make it grow.
    pub fn rotate(&mut self, orientation: super::Quaternion) {
        let mut v = self.vertices();

        let mut mat_model = super::Matrix4::one();
        mat_model = mat_model * orientation.to_mat4();

        for vertex in &mut v {
            let temp = mat_model * super::Vector4::new(vertex.x, vertex.y, vertex.z, 1.0);
            *vertex = super::Vector3::new(temp.x, temp.y, temp.z);
        }

        self.min = v[0];
        self.max = v[0];

        for vertex in &v {
            self.min = super::min(self.min, *vertex);
            self.max = super::max(self.max, *vertex);
        }
    }
}
