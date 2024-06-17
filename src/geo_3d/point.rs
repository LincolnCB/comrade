use std::ops::{
    Add, AddAssign,
    Sub, SubAssign,
};
use std::fmt;
use serde::{Serialize, Deserialize};

use crate::geo_3d::{GeoVector, Plane, Surface};

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

    /// Create a new zero point.
    pub fn zero() -> Self {
        Point{x: 0.0, y: 0.0, z: 0.0}
    }

    /// Get the distance between two points.
    pub fn distance(&self, other: &Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;

        (dx*dx + dy*dy + dz*dz).sqrt()
    }
    
    /// Get the index of the nearest point on the surface to this point.
    pub fn nearest_point_idx(&self, surface: &Surface) -> usize {
        let mut min_dist = std::f32::MAX;
        let mut min_point_idx = 0;
        for (idx, vertex) in surface.vertices.iter().enumerate() {
            let dist = self.distance(&vertex.point);
            if dist < min_dist {
                min_dist = dist;
                min_point_idx = idx;
            }
        }
        min_point_idx
    }
    
    /// Get the closest point on the surface to this point.
    pub fn nearest_point(&self, surface: &Surface) -> Point {
        surface.vertices[self.nearest_point_idx(surface)].point
    }

    /// Check if a point is in the triangular prism defined by extruding a face in both directions
    pub fn is_above_surface_face(&self, surface: &Surface, face_idx: usize) -> bool {
        let face = &surface.faces[face_idx];
        let normal = face.get_normal();

        let halfplane_sign = |p: &Point, p1: &Point, p2: &Point| {
            let v_edge = *p2 - *p1;
            let v_point = *p - *p1;
            let cross = v_edge.cross(&normal);
            cross.dot(&v_point) > 0.0
        };

        let sign_1 = halfplane_sign(self, &surface.vertices[face.vertices[0]].point, &surface.vertices[face.vertices[1]].point);
        let sign_2 = halfplane_sign(self, &surface.vertices[face.vertices[1]].point, &surface.vertices[face.vertices[2]].point);
        let sign_3 = halfplane_sign(self, &surface.vertices[face.vertices[2]].point, &surface.vertices[face.vertices[0]].point);

        sign_1 == sign_2 && sign_2 == sign_3
    }

    /// Project a point onto a triangular face
    pub fn project_to_surface_face(&self, surface: &Surface, face_idx: usize) -> Point {
        let face = &surface.faces[face_idx];
        let normal = face.get_normal();

        // Project the point onto the plane of the face
        let mut proj_point = *self - (*self - surface.vertices[face.vertices[0]].point).proj_onto(&normal);

        // For each edge, check if the point is outside the edge
        // If so, project the point onto the edge
        for i in 0..3 {
            let p1 = surface.vertices[face.vertices[i]].point;
            let p2 = surface.vertices[face.vertices[(i + 1) % 3]].point;
            let p3 = surface.vertices[face.vertices[(i + 2) % 3]].point;

            let edge = p2 - p1;
            let vec_to_point = proj_point - p1;
            let cross = edge.cross(&normal);
            if cross.dot(&vec_to_point).signum() != cross.dot(&(p3 - p1)).signum() {
                proj_point = proj_point - vec_to_point.proj_onto(&cross);
            }
        }

        proj_point
    }

    /// Reflect this point across a plane.
    pub fn reflect_across(&self, plane: &Plane) -> Point {
        let dist = plane.distance_to_point(self);
        let normal = plane.get_normal();
        *self - normal * 2.0 * dist
    }
}
impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let precision = f.precision().unwrap_or(3);
        write!(f, "({:.*}, {:.*}, {:.*})", precision, self.x, precision, self.y, precision, self.z)
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
impl Sub<&Surface> for &Point {
    type Output = GeoVector;

    fn sub(self, surface: &Surface) -> GeoVector {
        let mut proj_point = self.nearest_point(surface);

        for face_idx in 0..surface.faces.len() {
            let proj = self.project_to_surface_face(surface, face_idx);
            if proj.distance(self) < proj_point.distance(self) {
                proj_point = proj;
            }
        }

        *self - proj_point
    }
}
impl Sub<Plane> for Point {
    type Output = GeoVector;

    fn sub(self, rhs: Plane) -> GeoVector {
        let dist = rhs.distance_to_point(&self);
        rhs.get_normal() * dist
    }
}
impl std::convert::From<GeoVector> for Point {
    fn from(vector: GeoVector) -> Self {
        Point{
            x: vector.x,
            y: vector.y,
            z: vector.z,
        }
    }
}