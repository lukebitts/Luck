use std::ops::{Add, Mul};
use num::traits::{One, Zero};
use super::{Matrix4, Vector3, atan2, cross, cos, sin};

/// A [quaternion](https://en.wikipedia.org/wiki/Quaternion) type.
#[derive(Default, PartialEq, Debug, Copy, Clone)]
pub struct Quaternion {
    ///
    pub x: f32,
    ///
    pub y: f32,
    ///
    pub z: f32,
    ///
    pub w: f32,
}

impl Quaternion {
    /// Returns a new instance of a quaternion with the specified values.
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Quaternion {
            x: x,
            y: y,
            z: z,
            w: w,
        }
    }

    /// Converts a rotation vector3 into a quaternion.
    pub fn from_euler(v: Vector3<f32>) -> Self {
        // TODO: Should this receive parameters in degrees or radians
        let c = cos(v * 0.5);
        let s = sin(v * 0.5);

        let mut ret = Quaternion::zero();

        ret.w = c.x * c.y * c.z + s.x * s.y * s.z;
        ret.x = s.x * c.y * c.z - c.x * s.y * s.z;
        ret.y = c.x * s.y * c.z + s.x * c.y * s.z;
        ret.z = c.x * c.y * s.z - s.x * s.y * c.z;

        ret
    }

    /// Creates a rotation matrix from the quaternion.
    pub fn to_mat4(&self) -> Matrix4<f32> {
        let mut ret = Matrix4::<f32>::one();
        let q = self;

        ret.c0.x = 1.0 - 2.0 * (q.y).powi(2) - 2.0 * (q.z).powi(2);
        ret.c0.y = 2.0 * q.x * q.y + 2.0 * q.z * q.w;
        ret.c0.z = 2.0 * q.x * q.z - 2.0 * q.y * q.w;

        ret.c1.x = 2.0 * q.x * q.y - 2.0 * q.z * q.w;
        ret.c1.y = 1.0 - 2.0 * (q.x).powi(2) - 2.0 * (q.z).powi(2);
        ret.c1.z = 2.0 * q.y * q.z + 2.0 * q.x * q.w;

        ret.c2.x = 2.0 * q.x * q.z + 2.0 * q.y * q.w;
        ret.c2.y = 2.0 * q.y * q.z - 2.0 * q.x * q.w;
        ret.c2.z = 1.0 - 2.0 * (q.x).powi(2) - 2.0 * (q.y).powi(2);

        ret
    }

    /// Returns the quaternion rotation in euler angles.
    pub fn to_euler(&self) -> Vector3<f32> {
        // TODO: Should this return in degrees or radians
        Vector3::new(self.pitch(), self.yaw(), self.roll())
    }

    fn roll(&self) -> f32 {
        let q = *self;
        atan2(2.0 * (q.x * q.y + q.w * q.z), q.w * q.w + q.x * q.x - q.y * q.y - q.z * q.z)
    }

    fn pitch(&self) -> f32 {
        let q = *self;
        atan2(2.0 * (q.y * q.z + q.w * q.x), q.w * q.w - q.x * q.x - q.y * q.y + q.z * q.z)
    }

    fn yaw(&self) -> f32 {
        let q = *self;
        (-2.0 * (q.x * q.z - q.w * q.y)).asin()
    }
}

impl Zero for Quaternion {
    fn zero() -> Self {
        Quaternion::new(0.0, 0.0, 0.0, 1.0)
    }
    fn is_zero(&self) -> bool {
        Quaternion::zero() == *self
    }
}

impl One for Quaternion {
    fn one() -> Self {
        Quaternion::new(1.0, 1.0, 1.0, 1.0)
    }
}

impl Add for Quaternion {
    type Output = Quaternion;
    fn add(self, rhs: Quaternion) -> Quaternion {
        Quaternion::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z, self.w + rhs.w)
    }
}

impl Mul for Quaternion {
    type Output = Quaternion;
    fn mul(self, rhs: Quaternion) -> Quaternion {
        let mut ret = Quaternion::one();

        ret.w = (self.w * rhs.w) - (self.x * rhs.x) - (self.y * rhs.y) - (self.z * rhs.z);
        ret.x = (self.w * rhs.x) + (self.x * rhs.w) + (self.y * rhs.z) - (self.z * rhs.y);
        ret.y = (self.w * rhs.y) + (self.y * rhs.w) + (self.z * rhs.x) - (self.x * rhs.z);
        ret.z = (self.w * rhs.z) + (self.z * rhs.w) + (self.x * rhs.y) - (self.y * rhs.x);

        ret
    }
}

impl Mul<Vector3<f32>> for Quaternion {
    type Output = Vector3<f32>;
    fn mul(self, rhs: Vector3<f32>) -> Vector3<f32> {
        let quat_vector = Vector3::new(self.x, self.y, self.z);
        let uv = cross(quat_vector, rhs);
        let uuv = cross(quat_vector, uv);

        rhs + ((uv * self.w) + uuv) * 2f32
    }
}

#[cfg(test)]
mod test {
    use super::Quaternion;
    use super::super::Vector3;
    use num::traits::{One, Zero};

    #[test]
    fn conversion_operations() {
        let q = Quaternion::new(1.0, 0.0, 0.0, 1.0);
        assert_eq!(q.to_euler(), Vector3::new(1.5708, -0.0, 0.0));
    }

    #[test]
    fn num_operations() {
        assert_eq!(Quaternion::zero(), Quaternion::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(Quaternion::one(), Quaternion::new(1.0, 1.0, 1.0, 1.0));

        let q = Quaternion::one();
        assert_eq!(q + q, Quaternion::new(2.0, 2.0, 2.0, 2.0));

        let q = Quaternion::zero();
        assert_eq!(q * q, q);

        let q = Quaternion::new(1.0, 0.0, 0.0, 1.0);
        assert_eq!(q * q, Quaternion::new(2.0, 0.0, 0.0, 0.0));

        let q = Quaternion::new(0.0, 1.0, 0.0, 1.0);
        assert_eq!(q * q, Quaternion::new(0.0, 2.0, 0.0, 0.0));

        let q = Quaternion::new(0.0, 0.0, 1.0, 1.0);
        assert_eq!(q * q, Quaternion::new(0.0, 0.0, 2.0, 0.0));

        let q = Quaternion::new(1.0, 0.0, 0.0, 1.0);
        let v = Vector3::new(1.0, 1.0, 1.0);
        assert_eq!(q * v, Vector3::new(1.0, -3.0, 1.0));
    }

}
