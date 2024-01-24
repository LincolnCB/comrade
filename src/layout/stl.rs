use std::fs::OpenOptions;
use stl_io;

use crate::layout;
use layout::geo_3d::{
    Surface,
    Point,
};

/// Load a STL file from the inut path.
/// Uses the `stl_io` crate.
/// Returns a `ProcResult` with the `Surface` or an `Err`
pub fn load_stl(filename: &str) -> layout::ProcResult<Surface>{
    let mut file = OpenOptions::new().read(true).open(filename)?;
    let stl = stl_io::read_stl(&mut file)?;

    // Initialize the surface struct
    let mut surface = Surface{points: Vec::new(), area: 0.0};

    // First, copy all the points over
    for vertex in stl.vertices.into_iter() {
        surface.points.push(Point{
            x: vertex[0],
            y: vertex[1],
            z: vertex[2],
            adj: Vec::new(),
        });
    }

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
                    surface.points[point_index].adj.push(adj_index);
                }
            }
        }
    }

    // Sort and dedup the adjacent points
    for point in surface.points.iter_mut() {
        point.adj.sort();
        point.adj.dedup();
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

#[cfg(test)]
mod tests {

    use super::*;

    /// Test the example binary stl file.
    #[test]
    fn check_binary_stl() {
        let surface = load_stl("tests/data/tiny_cap.stl").unwrap();
        check_surface_point_count(&surface, 805);
        for point in surface.points.iter() {
            check_point_adj_sorted(point);
            check_point_adj_no_dup(point);
        }
    }

    /// Test the example ascii stl file.
    #[test]
    fn check_ascii_stl() {
        let surface = load_stl("tests/data/tiny_cap_ascii.stl").unwrap();
        check_surface_point_count(&surface, 805);
        for point in surface.points.iter() {
            check_point_adj_sorted(point);
            check_point_adj_no_dup(point);
        }
    }

    /// Test the remeshed stl file.
    #[test]
    fn check_remeshed_stl() {
        let surface = load_stl("tests/data/tiny_cap_remesh.stl").unwrap();
        check_surface_point_count(&surface, 4592);
        for point in surface.points.iter() {
            check_point_adj_sorted(point);
            check_point_adj_no_dup(point);
        }
    }

    /// Test the face area function.
    #[test] 
    fn check_face_area() {
        let p1 = Point::new(0.0, 0.0, 0.0);
        let p2 = Point::new(1.0, 0.0, 0.0);
        let p3 = Point::new(0.0, 1.0, 0.0);

        let true_area = 0.5;

        let face = stl_io::IndexedTriangle{vertices: [0, 1, 2], normal: stl_io::Vector::new([0.0, 0.0, 1.0] as [f32; 3])};
        let points = vec![p1, p2, p3];

        let area = face_area(&face, &points);
        assert!(area - true_area < 0.0001 * true_area);
    }

    /// Test the surface area function.
    #[test]
    fn check_surface_area() {
        let surface = load_stl("tests/data/tiny_cap.stl").unwrap();
        let true_area = 340.5;
        println!("Surface area: {}", surface.area);
        assert!(surface.area - true_area < 0.01 * true_area);
    }

    /// Test the surface area on the remeshed stl file.
    #[test]
    fn check_remeshed_surface_area() {
        let surface = load_stl("tests/data/tiny_cap_remesh.stl").unwrap();
        let true_area = 340.46;
        println!("Surface area: {}", surface.area);
        assert!(surface.area - true_area < 0.01 * true_area);
    }

    fn check_surface_point_count(surface: &Surface, expected_count: usize) {
        assert_eq!(surface.points.len(), expected_count);
    }

    fn check_point_adj_sorted(point: &Point) {
        let mut sorted_adj = point.adj.clone();
        sorted_adj.sort();
        assert_eq!(point.adj, sorted_adj);
    }

    fn check_point_adj_no_dup(point: &Point) {
        let mut no_dup = point.adj.clone();
        no_dup.dedup();
        assert_eq!(point.adj, no_dup);
    }
}
