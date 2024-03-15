use stl_io;

use crate::layout;
use crate::io;
use layout::geo_3d::{
    Surface,
    Point,
    GeoVector,
    NEWSurface,
    SurfaceVertex,
    SurfaceEdge,
    SurfaceFace,
};

/// Load a STL file from the inut path.
/// Uses the `stl_io` crate.
/// Returns a `ProcResult` with the `Surface` or an `Err`
pub fn load_stl(filename: &str) -> layout::ProcResult<Surface>{
    let mut file = match std::fs::OpenOptions::new().read(true).open(filename)
    {
        Ok(file) => file,
        Err(error) => {
            return Err(crate::io::IoError{file: filename.to_string(), cause: error}.into());
        },
    };
    let stl = match stl_io::read_stl(&mut file)
    {
        Ok(stl) => stl,
        Err(error) => {
            return Err(crate::io::IoError{file: filename.to_string(), cause: error}.into());
        },
    };

    // Initialize the surface struct
    let mut surface = Surface::empty();

    // First, copy all the points over
    for vertex in stl.vertices.into_iter() {
        surface.points.push(Point{
            x: vertex[0],
            y: vertex[1],
            z: vertex[2],
        });
    }

    // Track each point's adjacent points and normals
    let mut point_normals = vec![Vec::<GeoVector>::new(); surface.points.len()];
    surface.adj = vec![Vec::<usize>::new(); surface.points.len()];

    // Then, add the adjacent points
    for face in stl.faces.into_iter() {
        surface.area += face_area(&face, &surface.points);
        // For each point in the triangle:
        for i in 0..3 {
            // Find the point index
            let point_index = face.vertices[i];

            // For each other point in the triangle:
            for j in 0..3 {
                if j != i {
                    // Push the other point index to the adjacent list
                    let adj_index = face.vertices[j];
                    surface.adj[point_index].push(adj_index);
                }
            }

            // Add the normal to the point's normal list
            point_normals[point_index].push(GeoVector::new(face.normal[0], face.normal[1], face.normal[2]).normalize());
        }
    }

    // Average the point normals:
    for normal_list in point_normals.iter() {
        let mut avg_normal = GeoVector::new(0.0, 0.0, 0.0);
        for normal in normal_list.iter() {
            avg_normal += *normal;
        }
        avg_normal /= normal_list.len() as f32;
        surface.point_normals.push(avg_normal.normalize());
    }

    // Sort and dedup the adjacent points
    for adj_list in surface.adj.iter_mut() {
        adj_list.sort();
        adj_list.dedup();
    }

    Ok(surface)
}

// TODO: Update the STL loading to use the new Surface struct
// /// Load a STL file from the inut path.
// /// Uses the `stl_io` crate.
// /// Returns a `ProcResult` with the `Surface` or an `Err`
pub fn load_stl_new(filename: &str) -> layout::ProcResult<NEWSurface>{
    let mut file = io::open(filename)?;
    let stl = match stl_io::read_stl(&mut file)
    {
        Ok(stl) => stl,
        Err(error) => {
            return Err(io::IoError{file: filename.to_string(), cause: error}.into());
        },
    };

    // Initialize the surface struct
    let mut surface = NEWSurface::empty();

    // First, create vertices for each point
    for vertex in stl.vertices.into_iter() {
        surface.vertices.push(SurfaceVertex::new_from_point(
            Point{
                x: vertex[0],
                y: vertex[1],
                z: vertex[2],
            }
        ));
    }

    let mut edges = Vec::<SurfaceEdge>::new();

    // First, initialize all edges from the faces
    for tri_face in stl.faces.iter() {
        for i in 0..3 {
            let pid1 = tri_face.vertices[i];
            let pid2 = tri_face.vertices[(i + 1) % 3];
            let edge = SurfaceEdge::new([pid1, pid2]);
            edges.push(edge);
        }
    }

    // Sort and dedup them
    edges.sort_by(|a, b| a.vertices[0].cmp(&b.vertices[0]).then(a.vertices[1].cmp(&b.vertices[1])));
    edges.dedup();

    // Create a hashmap for the edge indices, so the faces and points can easily access them
    let mut edge_indices = std::collections::HashMap::<(usize, usize), usize>::new();
    for (i, edge) in edges.iter().enumerate() {
        edge_indices.insert((edge.vertices[0], edge.vertices[1]), i);
    }

    // Add faces to the surface, and add the faces to the edges
    for (face_id, tri_face) in stl.faces.into_iter().enumerate() {
        let mut face_vertices = Vec::<usize>::new();
        let mut face_edges = Vec::<usize>::new();
        for i in 0..3 {
            let pid1 = tri_face.vertices[i];
            face_vertices.push(pid1);
            let pid2 = tri_face.vertices[(i + 1) % 3];
            let edge_index = edge_indices.get(&(pid1, pid2)).unwrap();
            face_edges.push(*edge_index);
            if edges[*edge_index].adj_faces[0] == None {
                edges[*edge_index].adj_faces[0] = Some(face_id);
            } else if edges[*edge_index].adj_faces[1] == None {
                edges[*edge_index].adj_faces[1] = Some(face_id);
            } else {
                panic!("Edge {:?} has more than 2 faces!", edges[*edge_index]);
            }
        }
        let face_normal = GeoVector::new(tri_face.normal[0], tri_face.normal[1], tri_face.normal[2]).normalize();

        // Calculate the face area using Heron's formula
        let p1 = &surface.vertices[face_vertices[0]].point;
        let p2 = &surface.vertices[face_vertices[1]].point;
        let p3 = &surface.vertices[face_vertices[2]].point;
        let a = p1.distance(p2);
        let b = p2.distance(p3);
        let c = p3.distance(p1);
        let s = (a + b + c) / 2.0;
        let area = (s * (s - a) * (s - b) * (s - c)).sqrt();

        surface.faces.push(
            SurfaceFace{
                vertices: face_vertices,
                edges: face_edges,
                normal: face_normal,
                area,
            }
        );
    }

    // Add adjacent edges to the vertices
    for edge_index in 0..edges.len() {
        let edge = &edges[edge_index];
        for vid in 0..2 {
            let vertex = &mut surface.vertices[edge.vertices[vid]];
            vertex.adj_edges.push(edge_index);
        }
    }
    for vertex in surface.vertices.iter_mut() {
        vertex.adj_edges.sort();
        vertex.adj_edges.dedup();
    }

    // Add edges to the surface
    for edge in edges.into_iter() {
        surface.edges.push(edge);
    }

    Ok(surface)
}

/// Get the area of the triangle.
/// Uses Heron's formula.
fn face_area(face: &stl_io::IndexedTriangle, points: &Vec<Point>) -> f32 {
    let p1 = &points[face.vertices[0]];
    let p2 = &points[face.vertices[1]];
    let p3 = &points[face.vertices[2]];

    let a = p1.distance(p2);
    let b = p2.distance(p3);
    let c = p3.distance(p1);

    let s = (a + b + c) / 2.0;
    (s * (s - a) * (s - b) * (s - c)).sqrt()
}

// TODO: FIX TESTS
// #[cfg(test)]
// mod tests {

//     use super::*;

//     /// Test the example binary stl file.
//     #[test]
//     fn check_binary_stl() {
//         let surface = load_stl("tests/data/tiny_cap.stl").unwrap();
//         check_surface_point_count(&surface, 805);
//         for point in surface.points.iter() {
//             check_point_adj_sorted(point);
//             check_point_adj_no_dup(point);
//         }
//     }

//     /// Test the example ascii stl file.
//     #[test]
//     fn check_ascii_stl() {
//         let surface = load_stl("tests/data/tiny_cap_ascii.stl").unwrap();
//         check_surface_point_count(&surface, 805);
//         for point in surface.points.iter() {
//             check_point_adj_sorted(point);
//             check_point_adj_no_dup(point);
//         }
//     }

//     /// Test the remeshed stl file.
//     #[test]
//     fn check_remeshed_stl() {
//         let surface = load_stl("tests/data/tiny_cap_remesh.stl").unwrap();
//         check_surface_point_count(&surface, 4592);
//         for point in surface.points.iter() {
//             check_point_adj_sorted(point);
//             check_point_adj_no_dup(point);
//         }
//     }

//     /// Test the face area function.
//     #[test] 
//     fn check_face_area() {
//         let p1 = Point::new(0.0, 0.0, 0.0);
//         let p2 = Point::new(1.0, 0.0, 0.0);
//         let p3 = Point::new(0.0, 1.0, 0.0);

//         let true_area = 0.5;

//         let face = stl_io::IndexedTriangle{vertices: [0, 1, 2], normal: stl_io::Vector::new([0.0, 0.0, 1.0] as [f32; 3])};
//         let points = vec![p1, p2, p3];

//         let area = face_area(&face, &points);
//         assert!(area - true_area < 0.0001 * true_area);
//     }

//     /// Test the surface area function.
//     #[test]
//     fn check_surface_area() {
//         let surface = load_stl("tests/data/tiny_cap.stl").unwrap();
//         let true_area = 340.5;
//         println!("Surface area: {}", surface.area);
//         assert!(surface.area - true_area < 0.01 * true_area);
//     }

//     /// Test the surface area on the remeshed stl file.
//     #[test]
//     fn check_remeshed_surface_area() {
//         let surface = load_stl("tests/data/tiny_cap_remesh.stl").unwrap();
//         let true_area = 340.46;
//         println!("Surface area: {}", surface.area);
//         assert!(surface.area - true_area < 0.01 * true_area);
//     }

//     fn check_surface_point_count(surface: &Surface, expected_count: usize) {
//         assert_eq!(surface.points.len(), expected_count);
//     }

//     fn check_point_adj_sorted(point: &Point) {
//         let mut sorted_adj = point.adj.clone();
//         sorted_adj.sort();
//         assert_eq!(point.adj, sorted_adj);
//     }

//     fn check_point_adj_no_dup(point: &Point) {
//         let mut no_dup = point.adj.clone();
//         no_dup.dedup();
//         assert_eq!(point.adj, no_dup);
//     }
// }
