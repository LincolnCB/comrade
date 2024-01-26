use crate::{
    layout,
    mesh,
    args,
};
use mesh::methods;
use layout::geo_3d::*;

use std::f32::consts::PI;
use std::fs::OpenOptions;

/// STL Polygons Method struct.
/// This struct contains all the parameters for the STL Polygons meshing method.
#[derive(Debug)]
#[allow(dead_code)]
pub struct Method {
    /// Arguments for the STL Polygons method.
    method_args: MethodArgs,
}

/// TODO: Expand to actually parse argfile
#[derive(Debug)]
struct MethodArgs {
    radius: f32,
    poly_num: usize,
}

impl Method {
    pub fn new() -> args::ProcResult<Self> {
        Ok(Method{method_args: MethodArgs{
            radius: 1.0,
            // TODO: Make sure polynum is over 3
            poly_num: 8,
        }})
    }
}

impl methods::MeshMethod for Method {
    /// Get the name of the meshing method.
    fn get_method_name(&self) -> String {
        "STL Polygons".to_string()
    }

    /// Parse the meshing method argument file
    #[allow(unused_variables)]
    fn parse_method_args(&mut self, arg_file: &str) -> args::ProcResult<()>{
        // TODO: Expand
        Ok(())
    }

    /// Run the meshing process with the given arguments.
    /// Uses the `mesh` and `layout` modules.
    fn save_mesh(&self, layout: &layout::Layout, output_path: &str) -> mesh::ProcResult<()> {
        // Final check out output path
        if !output_path.ends_with(".stl") {
            mesh::err_str("BUG: Mesh output path must end with .stl -- somehow got to the meshing stage without that!!")?;
        }
        
        // Mesh each coil
        for (coil_n, coil) in layout.coils.iter().enumerate() {
            println!("Coil {}...", coil_n);

            // Initialize the triangle list
            let mut triangles = Vec::<stl_io::Triangle>::new();

            // Create the corner slice polygons
            let mut corner_slices = Vec::<Vec::<Point>>::new();
            for point in coil.points.iter() {
                let mut corner_slice = Vec::new();

                if point.adj.len() != 2 {
                    mesh::err_str(&format!("Coil point {point} has wrong number of adjacent points {}", point.adj.len()))?;
                }

                let prev_point = &coil.points[point.adj[0]];
                let next_point = &coil.points[point.adj[1]];

                let mut vec1 = GeoVector::new_from_points(prev_point, point);
                let mut vec2 = GeoVector::new_from_points(next_point, point);

                vec1.normalize();
                vec2.normalize();

                let mut out_vec = vec1 + vec2;
                out_vec.normalize();

                let mut up_vec = vec1.cross(&vec2);
                up_vec.normalize();

                // Put the polygon points around the plane given by the point and the out_vec/up_vec
                for i in 0..self.method_args.poly_num {
                    let angle = (i as Angle) * 2.0 * PI / (self.method_args.poly_num as Angle);
                    let mut poly_point = point.dup() + out_vec * angle.cos() * self.method_args.radius + up_vec * angle.sin() * self.method_args.radius;
                    poly_point.adj = vec![(i + self.method_args.poly_num - 1) % self.method_args.poly_num, (i + 1) % self.method_args.poly_num];
                    corner_slice.push(poly_point);
                }

                corner_slices.push(corner_slice);
            }

            // For each corner, mesh the section to the next corner
            for (slice_id, point) in coil.points.iter().enumerate() {
                let next_slice_id = point.adj[1];
                let slice = &corner_slices[slice_id];
                let next_slice = &corner_slices[next_slice_id];

                if slice.len() != next_slice.len() {
                    mesh::err_str(&format!("BUG: Coil corner {0} has a different number of points ({1}) than the next {2} ({3})", 
                        slice_id, slice.len(), next_slice_id, next_slice.len()))?;
                }
                
                for (v_n, v0) in slice.iter().enumerate() {
                    let v1 = &slice[v0.adj[1]];
                    let w0 = &next_slice[v_n];
                    let w1 = &next_slice[w0.adj[1]];

                    let mut n0 = GeoVector::new_from_points(v0, v1).cross(&GeoVector::new_from_points(v0, w0));
                    let mut n1 = GeoVector::new_from_points(w0, v1).cross(&GeoVector::new_from_points(w0, w1));

                    n0.normalize();
                    n1.normalize();

                    triangles.push(stl_io::Triangle{
                        normal: stl_io::Normal::new([n0.x, n0.y, n0.z]),
                        vertices: [
                            stl_io::Vertex::new([v0.x, v0.y, v0.z]),
                            stl_io::Vertex::new([v1.x, v1.y, v1.z]),
                            stl_io::Vertex::new([w0.x, w0.y, w0.z]),
                        ]
                    });

                    triangles.push(stl_io::Triangle{
                        normal: stl_io::Normal::new([n1.x, n1.y, n1.z]),
                        vertices: [
                            stl_io::Vertex::new([v1.x, v1.y, v1.z]),
                            stl_io::Vertex::new([w1.x, w1.y, w1.z]),
                            stl_io::Vertex::new([w0.x, w0.y, w0.z]),
                        ]
                    });
                }
            }

            // Save each coil to a separate file
            let numbered_output_path = output_path.replace(".stl", &format!("_c{}.stl", coil_n));
            println!("Saving coil {} to {}...", coil_n, numbered_output_path);
            let mut file = match OpenOptions::new().write(true).create(true).open(&numbered_output_path)
            {
                Ok(file) => file,
                Err(error) => {
                    return Err(crate::io::IoError{file: numbered_output_path.to_string(), cause: error}.into());
                },
            };
            match stl_io::write_stl(&mut file, triangles.iter())
            {
                Ok(_) => (),
                Err(error) => {
                    return Err(crate::io::IoError{file: numbered_output_path.to_string(), cause: error}.into());
                },
            };
        }

        Ok(())
    }
}


