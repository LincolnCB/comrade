use std::ops::{
    Add, AddAssign,
    Sub, SubAssign,
    Mul, MulAssign,
    Div, DivAssign,
};
use std::fmt;
use serde::{Serialize, Deserialize};

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
        let normal = face.normal;

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
        let normal = face.normal;

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
        let normal = plane.normal;
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
        rhs.normal * dist
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

    /// Create a new zero vector.
    pub fn zero() -> Self {
        GeoVector{x: 0.0, y: 0.0, z: 0.0}
    }

    /// Normailze in place
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

    /// Get the magnitude of the vector.
    pub fn norm(&self) -> f32 {
        (self.x*self.x + self.y*self.y + self.z*self.z).sqrt()
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

/// An updated surface
// TODO: expand docs
#[derive(Debug)]
pub struct Surface {
    pub vertices: Vec<SurfaceVertex>,
    pub edges: Vec<SurfaceEdge>,
    pub faces: Vec<SurfaceFace>,
}
impl Surface {
    pub fn empty() -> Self {
        Surface{
            vertices: Vec::new(),
            edges: Vec::new(),
            faces: Vec::new(),
        }
    }

    pub fn get_boundary_vertex_indices(&self) -> Vec<usize> {
        let mut boundary_vertex_indices = Vec::new();

        for edge in self.edges.iter() {
            if edge.adj_faces.iter().any(|f| f.is_none()) {
                boundary_vertex_indices.extend_from_slice(&edge.vertices);
            }
        }

        boundary_vertex_indices.sort();
        boundary_vertex_indices.dedup();
        boundary_vertex_indices
    }

    /// Trim the surface by a plane.
    /// Returns the new surface and the indices of the vertices on the cut boundary.
    pub fn trim_by_plane(&self, plane: &Plane, flatten_cut: bool) -> (Self, Vec<usize>) {
        let mut new_surface = Surface::empty();

        // Add vertices
        let mut vertex_map = Vec::new();
        for vertex in self.vertices.iter() {
            if plane.distance_to_point(&vertex.point) >= 0.0 {
                let new_vertex_idx = new_surface.vertices.len();
                let mut new_vertex = SurfaceVertex::new_from_point(vertex.point);
                new_vertex.normal = vertex.normal;
                new_surface.vertices.push(new_vertex);
                vertex_map.push(Some(new_vertex_idx));
            } else {
                vertex_map.push(None);
            }
        }

        // Add edges
        let mut edge_map = Vec::new();
        for edge in self.edges.iter() {
            if let [Some(v1), Some(v2)] = [vertex_map[edge.vertices[0]], vertex_map[edge.vertices[1]]] {
                let new_edge = SurfaceEdge::new([v1, v2]);
                let new_edge_idx = new_surface.edges.len();
                new_surface.edges.push(new_edge);
                edge_map.push(Some(new_edge_idx));
            } else {
                edge_map.push(None);
            }
        }

        // Track which new vertices are on the cut boundary
        let mut cut_boundary_vertex_indices = Vec::new();

        // Add faces
        for face in self.faces.iter() {
            let mut new_face_vertices: [usize; 3] = [0; 3];
            let mut new_face_edges: [usize; 3] = [0; 3];

            let mut vertices_inside = 0;
            for (idx, vertex_idx) in face.vertices.iter().enumerate() {
                if let Some(new_vertex_idx) = vertex_map[*vertex_idx] {
                    new_face_vertices[idx] = new_vertex_idx;
                    let next_vertex_idx = face.vertices[(idx + 1) % 3];
                    let edge_idx = self.get_edge_idx(*vertex_idx, next_vertex_idx);
                    if let Some(new_edge_idx) = edge_map[edge_idx] {
                        new_face_edges[idx] = new_edge_idx;
                    }
                    vertices_inside += 1;
                }
            }

            // For faces that are removed from the cut, mark the remaining vertices as cut boundary vertices
            if vertices_inside > 0 && vertices_inside < 3 {
                for vertex_idx in face.vertices.iter() {
                    if let Some(new_vertex_idx) = vertex_map[*vertex_idx] {
                        cut_boundary_vertex_indices.push(new_vertex_idx);
                    }
                }
            }

            // Add the remaining faces
            if vertices_inside == 3 {
                let new_face = SurfaceFace{
                    vertices: new_face_vertices,
                    edges: new_face_edges,
                    normal: face.normal,
                    area: face.area,
                };
                new_surface.faces.push(new_face);
            }
        }

        // Update adjacencies
        for (new_edge_idx, new_edge) in new_surface.edges.iter().enumerate() {
            for vertex_idx in new_edge.vertices.iter() {
                new_surface.vertices[*vertex_idx].adj_edges.push(new_edge_idx);
            }
        }
        for (new_face_idx, new_face) in new_surface.faces.iter().enumerate() {
            for vertex_idx in new_face.vertices.iter() {
                new_surface.vertices[*vertex_idx].adj_faces.push(new_face_idx);
            }
            for edge_idx in new_face.edges.iter() {
                new_surface.edges[*edge_idx].adj_faces[0] = Some(new_face_idx);
            }
        }

        // Sort adjacencies
        for vertex in new_surface.vertices.iter_mut() {
            vertex.adj_edges.sort();
            vertex.adj_faces.sort();
        }
        for edge in new_surface.edges.iter_mut() {
            edge.adj_faces.sort();
        }
        for face in new_surface.faces.iter_mut() {
            face.edges.sort();
        }

        // Flatten the cut
        if flatten_cut {
            for vertex_idx in cut_boundary_vertex_indices.iter() {
                let vertex = &mut new_surface.vertices[*vertex_idx];
                let new_point = plane.project_point(&vertex.point);
                vertex.point = new_point;
                vertex.normal = vertex.normal.rej_onto(&plane.normal);
            }

            // Update the normals
            for face in new_surface.faces.iter_mut() {
                let p1 = new_surface.vertices[face.vertices[0]].point;
                let p2 = new_surface.vertices[face.vertices[1]].point;
                let p3 = new_surface.vertices[face.vertices[2]].point;
                let new_normal = (p2 - p1).cross(&(p3 - p1)).normalize();
                face.normal = new_normal;
            }

            // Update the areas
            for face in new_surface.faces.iter_mut() {
                let p1 = new_surface.vertices[face.vertices[0]].point;
                let p2 = new_surface.vertices[face.vertices[1]].point;
                let p3 = new_surface.vertices[face.vertices[2]].point;
                let a = p1.distance(&p2);
                let b = p2.distance(&p3);
                let c = p3.distance(&p1);
                let s = (a + b + c) / 2.0;
                let area = (s * (s - a) * (s - b) * (s - c)).sqrt();
                face.area = area;
            }
        }

        (new_surface, cut_boundary_vertex_indices)
    }

    /// Get the index of the edge between two vertices.
    fn get_edge_idx(&self, v1: usize, v2: usize) -> usize {
        for edge_idx in self.vertices[v1].adj_edges.iter() {
            let edge = &self.edges[*edge_idx];
            if edge.vertices.contains(&v2) {
                return *edge_idx;
            }
        }
        panic!("Edge not found between vertices {} and {}", v1, v2);
    }
}

#[derive(Debug)]
pub struct SurfaceVertex {
    pub point: Point,
    pub normal: GeoVector,
    pub adj_edges: Vec<usize>,
    pub adj_faces: Vec<usize>,
}
impl SurfaceVertex {
    pub fn new_from_point(point: Point) -> Self {
        SurfaceVertex{
            point,
            normal: GeoVector::zero(),
            adj_edges: Vec::new(),
            adj_faces: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SurfaceEdge {
    pub vertices: [usize; 2],
    pub adj_faces: [Option::<usize>; 2],
}
impl SurfaceEdge {
    pub fn new(vertices: [usize; 2]) -> Self {
        let mut vertices = vertices;
        vertices.sort();
        assert!(vertices[0] != vertices[1]);
        SurfaceEdge{    
            vertices,
            adj_faces: [None, None],
        }
    }
}

#[derive(Debug)]
pub struct SurfaceFace {
    pub vertices: [usize; 3],
    pub edges: [usize; 3],
    pub normal: GeoVector,
    pub area: f32,
}
