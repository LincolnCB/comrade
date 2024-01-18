use std::fs::OpenOptions;
use stl_io;

use super::{
    Surface,
    Point,
};

/// Load a STL file from the inut path.
/// Uses the `stl_io` crate.
/// Returns a `Result` with the `Surface` or an `Err`
pub fn load_stl(filename: &str) -> crate::Result<Surface>{
    let mut file = OpenOptions::new().read(true).open(filename)?;
    let stl = stl_io::read_stl(&mut file)?;

    // Initialize the surface struct
    let mut surface = Surface{points: Vec::new()};

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

#[cfg(test)]
mod tests {

    use super::*;

    /// Test the example binary stl file.
    #[test]
    fn check_binary_stl() {
        let surface = load_example_binary_stl();
        check_surface_point_count(&surface, 805);
        for point in surface.points.iter() {
            check_point_adj_sorted(point);
            check_point_adj_no_dup(point);
        }
    }

    /// Test the example ascii stl file.
    #[test]
    fn check_ascii_stl() {
        let surface = load_example_ascii_stl();
        check_surface_point_count(&surface, 805);
        for point in surface.points.iter() {
            check_point_adj_sorted(point);
            check_point_adj_no_dup(point);
        }
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

    fn load_example_binary_stl()-> Surface {
        load_stl("tests/data/tiny_cap.stl").unwrap()
    }

    fn load_example_ascii_stl() -> Surface {
        load_stl("tests/data/tiny_cap_ascii.stl").unwrap()
    }
}
