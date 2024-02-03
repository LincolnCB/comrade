use std::ops::{
    Add, AddAssign,
    Sub, SubAssign,
    Mul, MulAssign,
    Div, DivAssign,
};
use std::fmt;
use serde::{Serialize, Deserialize};

/// The substrate surface.
/// Contains a list of points.
#[derive(Debug)]
pub struct Surface {
    pub points: Vec<Point>,
    pub adj: Vec<Vec<usize>>,
    pub area: f32,
    pub point_normals: Vec<GeoVector>,
}
impl Surface {
    pub fn empty() -> Self {
        Surface{
            points: Vec::new(),
            adj: Vec::new(),
            area: 0.0,
            point_normals: Vec::new(),
        }
    }
}

/// Angle type (alias for f32).
pub type Angle = f32;

/// A point in 3D space.
/// Contains the coordinates of the point.
/// Has basic math support for adding and subtracting vectors.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Point {
    /// Create a new point.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Point{x, y, z}
    }

    /// Get the distance between two points.
    pub fn distance(&self, other: &Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;

        (dx*dx + dy*dy + dz*dz).sqrt()
    }
}
impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}
impl Add<GeoVector> for Point {
    type Output = Self;

    fn add(self, rhs: GeoVector) -> Self {
        Point{
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}
impl AddAssign<GeoVector> for Point {
    fn add_assign(&mut self, rhs: GeoVector) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}
impl Sub<GeoVector> for Point {
    type Output = Self;

    fn sub(self, rhs: GeoVector) -> Self {
        Point{
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}
impl SubAssign<GeoVector> for Point {
    fn sub_assign(&mut self, rhs: GeoVector) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}
impl Sub<Point> for Point {
    type Output = GeoVector;

    fn sub(self, rhs: Self) -> GeoVector {
        GeoVector{
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}
impl Sub<&Point> for &Point {
    type Output = GeoVector;

    fn sub(self, rhs: &Point) -> GeoVector {
        GeoVector{
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

/// A vector in 3D space.
/// Used for the normal vector of a point.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
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

    /// Normailze in place
    pub fn normalize(&self) -> Self {
        let mag = self.mag();
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

    /// Get the magnitude of the vector.
    pub fn mag(&self) -> f32 {
        (self.x*self.x + self.y*self.y + self.z*self.z).sqrt()
    }

    /// Get the angle between two vectors.
    pub fn angle_to(&self, other: &GeoVector) -> Angle {
        let dot = self.dot(other);
        let mag = self.mag() * other.mag();
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
        let mag = other.mag();
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
impl fmt::Display for GeoVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}
