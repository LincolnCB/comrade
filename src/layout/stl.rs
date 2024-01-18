use std::fs::OpenOptions;
use stl_io;

use super::{
    Surface,
    Point,
};

/// Load a STL file from the inut path.
/// Uses the `stl_io` crate.
/// Returns a `Result` with the `Surface` or an `Err`
pub fn load_stl() -> crate::Result<Surface>{
    let mut file = OpenOptions::new().read(true).open("mesh.stl")?;
    let stl = stl_io::read_stl(&mut file)?;
    let _ = stl.validate()?;

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

    Ok(surface)     
}
