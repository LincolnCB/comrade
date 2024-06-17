use std::fmt;
use serde::{Serialize, Deserialize};

use crate::geo_3d::{Point, GeoVector};

/// A plane in 3D space.
/// Contains a normal vector and an offset.
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Plane {
    normal: GeoVector,
    pub offset: f32,
}
impl Plane {
    /// Create a new plane.
    pub fn from_normal_and_offset(normal: GeoVector, offset: f32) -> Self {
        let normal = normal.normalize();
        Plane{normal, offset}
    }

    /// Create a new plane from a normal vector and a point.
    pub fn from_normal_and_point(normal: GeoVector, point: Point) -> Self {
        let normal = normal.normalize();
        let offset = normal.dot(&point.into());
        Plane{normal, offset}
    }

    /// Create a new plane from three points.
    pub fn from_points(p1: Point, p2: Point, p3: Point) -> Self {
        let normal = (p2 - p1).cross(&(p3 - p1)).normalize();
        let offset = normal.dot(&p1.into());
        Plane{normal, offset}
    }

    /// Get the normal vector of the plane. Guaranteed to be normalized.
    pub fn get_normal(&self) -> GeoVector {
        self.normal
    }

    /// Get the distance from a point to the plane.
    pub fn distance_to_point(&self, point: &Point) -> f32 {
        self.normal.dot(&(*point).into()) - self.offset
    }

    /// Get the projection of a point onto the plane.
    pub fn project_point(&self, point: &Point) -> Point {
        *point - self.normal * self.distance_to_point(point)
    }
}
impl fmt::Display for Plane {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Plane: normal={}, offset={}", self.normal, self.offset)
    }
}