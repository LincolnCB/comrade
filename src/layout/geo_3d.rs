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
    
    /// Get the index of the nearest point on the surface to this point.
    pub fn nearest_point_idx(&self, surface: &Surface) -> usize {
        let mut min_dist = std::f32::MAX;
        let mut min_point_idx = 0;
        for i in 0..surface.points.len() {
            let dist = self.distance(&surface.points[i]);
            if dist < min_dist {
                min_dist = dist;
                min_point_idx = i;
            }
        }
        min_point_idx
    }
    
    /// Get the closest point on the surface to this point.
    pub fn nearest_point(&self, surface: &Surface) -> Point {
        surface.points[self.nearest_point_idx(surface)]
    }

    /// Project this point onto the nearest face of the surface.
    pub fn project_to_surface_face(&self, surface: &Surface) -> Point {
        *self - (self - surface)
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
impl Sub<&Surface> for &Point {
    type Output = GeoVector;

    fn sub(self, surface: &Surface) -> GeoVector {
        let min_point_idx = self.nearest_point_idx(surface);

        // Get the point normal and the distance rejection vector
        let min_point = surface.points[min_point_idx];
        let min_point_normal = surface.point_normals[min_point_idx];
        let vec_to_point = *self - min_point;
        let rej = vec_to_point.rej_onto(&min_point_normal).normalize();

        // Find the two adjacent points most aligned with the rejection vector
        let mut max_dot_1 = std::f32::MIN;
        let mut max_dot_2 = std::f32::MIN;
        let mut adj_point_1_idx = 0;
        let mut adj_point_2_idx = 0;
        for i in 0..surface.adj[min_point_idx].len() {
            let adj_point = surface.points[surface.adj[min_point_idx][i]];
            let adj_vec = adj_point - min_point;
            let dot = adj_vec.rej_onto(&min_point_normal).normalize().dot(&rej);
            if dot > max_dot_1 {
                max_dot_2 = max_dot_1;
                adj_point_2_idx = adj_point_1_idx;
                max_dot_1 = dot;
                adj_point_1_idx = i;
            }
            else if dot > max_dot_2 {
                max_dot_2 = dot;
                adj_point_2_idx = i;
            }
        }

        // Get the face normal
        let adj_point_1 = surface.points[surface.adj[min_point_idx][adj_point_1_idx]];
        let adj_point_2 = surface.points[surface.adj[min_point_idx][adj_point_2_idx]];
        let side_1 = adj_point_1 - min_point;
        let side_2 = adj_point_2 - min_point;
        let face_normal = side_1.cross(&side_2).normalize();

        vec_to_point.proj_onto(&face_normal)
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

    /// Rotate around another vector by an angle.
    pub fn rotate_around(&self, axis: &GeoVector, angle: Angle) -> GeoVector {
        let c = angle.cos();
        let s = angle.sin();
        let cross = axis.cross(&self);

        *self * c + cross * s + *axis * axis.dot(&self) * (1.0 - c)
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
