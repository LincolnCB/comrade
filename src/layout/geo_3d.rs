use std::ops::{
    Add,
    Sub,
};
use std::fmt;

/// The substrate surface.
/// Contains a list of points.
#[derive(Debug)]
pub struct Surface {
    pub points: Vec<Point>,
    pub area: f32,
}

/// A point in 3D space.
/// Contains the coordinates and a list of adjacent points.
/// The adjacent points are stored as indices in the `Surface` struct.
/// Adjacent points are found from the triangles in the STL file.
#[derive(Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub adj: Vec<usize>,
    // TODO: Maybe include averaged normal vector? Maybe one point per triangle?
}

impl Point {
    /// Create a new point.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Point{x, y, z, adj: Vec::new()}
    }

    /// Create a duplicate of the point, with no adjacent points.
    pub fn dup(&self) -> Self {
        Point{x: self.x, y: self.y, z: self.z, adj: Vec::new()}
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

/// Angle type (alias for f32).
pub type Angle = f32;

/// A vector in 3D space.
/// Used for the normal vector of a point.
#[derive(Debug)]
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

    /// Create a new vector from two points.
    pub fn new_from_points(p1: &Point, p2: &Point) -> Self {
        GeoVector{
            x: p2.x - p1.x,
            y: p2.y - p1.y,
            z: p2.z - p1.z,
        }
    }

    /// Normailze in place
    pub fn normalize(&mut self) {
        let mag = self.mag();
        self.x /= mag;
        self.y /= mag;
        self.z /= mag;
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

impl fmt::Display for GeoVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}
