use std::ops::{
    Add, AddAssign,
    Sub, SubAssign,
    Mul, MulAssign,
    Div, DivAssign,
};
use std::fmt;
use serde::{Serialize, Deserialize};

use crate::geo_3d::{Angle, Point};

/// A vector in 3D space.
/// Used for the normal vector of a point.
#[derive(Debug, Clone, Copy)]
#[derive(Serialize, Deserialize)]
pub struct GeoVector {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl GeoVector {
    /// Create a new vector.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        GeoVector{x, y, z}
    }

    /// Create a new zero vector.
    pub fn zero() -> Self {
        GeoVector{x: 0.0, y: 0.0, z: 0.0}
    }

    /// Normalize and return a new vector.
    pub fn normalize(&self) -> Self {
        let mag = self.norm();
        GeoVector{
            x: self.x / mag,
            y: self.y / mag,
            z: self.z / mag,
        }
    }

    /// Get the dot product of two vectors.
    pub fn dot(&self, other: &GeoVector) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Get the cross product of two vectors.
    pub fn cross(&self, other: &GeoVector) -> GeoVector {
        GeoVector{
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Get the magnitude squared of the vector.
    pub fn norm_sq(&self) -> f32 {
        self.x*self.x + self.y*self.y + self.z*self.z
    }

    /// Get the magnitude of the vector.
    pub fn norm(&self) -> f32 {
        self.norm_sq().sqrt()
    }

    /// Get the angle between two vectors.
    pub fn angle_to(&self, other: &GeoVector) -> Angle {
        let dot = self.dot(other);
        let mag = self.norm() * other.norm();
        // Catch float errors when vectors are exactly aligned
        if (dot / mag) > 1.0 {
            return 0.0;
        }
        if (dot / mag) < -1.0 {
            return std::f32::consts::PI;
        }
        (dot / mag).acos()
    }

    /// Get the vector projection of `self` onto `other`.
    pub fn proj_onto(&self, other: &GeoVector) -> GeoVector {
        let dot = self.dot(other);
        let mag = other.norm();
        GeoVector{
            x: dot * other.x / mag,
            y: dot * other.y / mag,
            z: dot * other.z / mag,
        }
    }

    /// Get the vector rejection of `self` onto `other`.
    pub fn rej_onto(&self, other: &GeoVector) -> GeoVector {
        *self - self.proj_onto(other)
    }

    /// Rotate around another vector by an angle.
    pub fn rotate_around(&self, axis: &GeoVector, angle: Angle) -> GeoVector {
        let c = angle.cos();
        let s = angle.sin();
        let cross = axis.cross(&self);

        *self * c + cross * s + *axis * axis.dot(&self) * (1.0 - c)
    }

    /// Reflect a vector across a normal vector.
    pub fn reflect_across(&self, normal: &GeoVector) -> GeoVector {
        let normal = normal.normalize();
        *self - normal * 2.0 * normal.dot(self)
    }

    /// Construct an xhat vector.
    pub fn xhat() -> Self {
        GeoVector{x: 1.0, y: 0.0, z: 0.0}
    }

    /// Construct a yhat vector.
    pub fn yhat() -> Self {
        GeoVector{x: 0.0, y: 1.0, z: 0.0}
    }

    /// Construct a zhat vector.
    pub fn zhat() -> Self {
        GeoVector{x: 0.0, y: 0.0, z: 1.0}
    }

    /// Check if any of the components are NaN.
    pub fn has_nan(&self) -> bool {
        self.x.is_nan() || self.y.is_nan() || self.z.is_nan()
    }
}
impl Add for GeoVector {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        GeoVector{
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}
impl AddAssign for GeoVector {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}
impl Sub for GeoVector {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        GeoVector{
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}
impl SubAssign for GeoVector {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}
impl Mul<GeoVector> for f32 {
    type Output = GeoVector;

    fn mul(self, other: GeoVector) -> GeoVector {
        GeoVector{
            x: self * other.x,
            y: self * other.y,
            z: self * other.z,
        }
    }
}
impl Mul<f32> for GeoVector {
    type Output = GeoVector;

    fn mul(self, other: f32) -> GeoVector {
        GeoVector{
            x: self.x * other,
            y: self.y * other,
            z: self.z * other,
        }
    }
}
impl MulAssign<f32> for GeoVector {
    fn mul_assign(&mut self, other: f32) {
        self.x *= other;
        self.y *= other;
        self.z *= other;
    }
}
impl Div<f32> for GeoVector {
    type Output = GeoVector;

    fn div(self, other: f32) -> GeoVector {
        GeoVector{
            x: self.x / other,
            y: self.y / other,
            z: self.z / other,
        }
    }
}
impl DivAssign<f32> for GeoVector {
    fn div_assign(&mut self, other: f32) {
        self.x /= other;
        self.y /= other;
        self.z /= other;
    }
}
impl std::ops::Neg for GeoVector {
    type Output = GeoVector;

    fn neg(self) -> GeoVector {
        GeoVector{
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}
impl std::convert::From<Point> for GeoVector {
    fn from(point: Point) -> Self {
        GeoVector{
            x: point.x,
            y: point.y,
            z: point.z,
        }
    }
}
impl fmt::Display for GeoVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let precision = f.precision().unwrap_or(3);
        write!(f, "({:.*}, {:.*}, {:.*})", precision, self.x, precision, self.y, precision, self.z)
    }
}
