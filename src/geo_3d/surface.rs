use crate::geo_3d::{Point, GeoVector, Plane};

/// A surface in 3D space. Contains vertices, edges, and faces, linked to each other.
#[derive(Debug, Clone)]
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
                vertex.normal = vertex.normal.rej_onto(&plane.get_normal());
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

#[derive(Debug, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
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

#[derive(Debug, Clone)]
pub struct SurfaceFace {
    pub vertices: [usize; 3],
    pub edges: [usize; 3],
    normal: GeoVector,
    pub area: f32,
}
impl SurfaceFace {
    pub fn new(vertices: [usize; 3], edges: [usize; 3], normal: GeoVector, area: f32) -> Self {
        SurfaceFace{
            vertices,
            edges,
            normal: normal.normalize(),
            area,
        }
    }

    /// Get the normal vector of the face. Normal vectors are private to guarantee that they are normalized.
    pub fn get_normal(&self) -> GeoVector {
        self.normal
    }
}
