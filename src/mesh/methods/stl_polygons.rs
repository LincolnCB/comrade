use crate::{
    layout,
    mesh,
    args,
};
use mesh::methods;
use layout::geo_3d::*;

use serde::{Serialize, Deserialize};
use std::fs::OpenOptions;
use std::f32::consts::PI;

/// STL Polygons Method struct.
/// This struct contains all the parameters for the STL Polygons meshing method.
#[derive(Debug)]
pub struct Method {
    /// Arguments for the STL Polygons method.
    method_args: MethodArgs,
}
impl Method {
    pub fn new() -> args::ProcResult<Self> {
        Ok(Method{method_args: MethodArgs::default()})
    }
}

/// Deserializer from yaml arg file
#[derive(Debug, Serialize, Deserialize)]
struct MethodArgs {
    #[serde(default = "MethodArgs::default_radius", alias = "wire_radius")]
    radius: f32,
    #[serde(default = "MethodArgs::default_poly_num")]
    poly_num: usize,
}
impl MethodArgs {
    pub fn default_radius() -> f32 {
        0.3
    }
    pub fn default_poly_num() -> usize {
        8
    }
    pub fn default() -> Self {
        MethodArgs{
            radius: Self::default_radius(),
            poly_num: Self::default_poly_num(),
        }
    }
}

impl methods::MeshMethod for Method {
    /// Get the name of the meshing method.
    fn get_method_name(&self) -> String {
        "STL Polygons".to_string()
    }

    /// Parse the meshing method argument file
    fn parse_method_args(&mut self, arg_file: &str) -> args::ProcResult<()>{
        let f = crate::io::open(arg_file)?;
        self.method_args = serde_yaml::from_reader(f)?;
        Ok(())
    }

    /// Run the meshing process with the given arguments.
    /// Uses the `mesh` and `layout` modules.
    fn save_mesh(&self, layout: &layout::Layout, output_path: &str) -> mesh::ProcResult<()> {
        // Final check out output path
        if !output_path.ends_with(".stl") {
            mesh::err_str("BUG: Mesh output path must end with .stl -- somehow got to the meshing stage without that!!")?;
        }
        
        let mut full_triangles = Vec::<stl_io::Triangle>::new();

        // Mesh each coil
        for (coil_n, coil) in layout.coils.iter().enumerate() {
            println!("Coil {}...", coil_n);

            // Initialize the triangle list
            let mut triangles = Vec::<stl_io::Triangle>::new();

            // Create the corner slice polygons
            let mut corner_slices = Vec::<Vec::<Point>>::new();
            for coil_vertex in coil.vertices.iter() {
                let mut corner_slice = Vec::new();

                let point = coil_vertex.point;

                let up_vec = coil_vertex.normal.normalize();
                let out_vec = (point - coil.center).rej_onto(&up_vec).normalize();

                // Put the polygon points around the plane given by the point and the out_vec/up_vec
                for i in 0..self.method_args.poly_num {
                    let angle = 2.0 * PI * (i as Angle - 0.5) / (self.method_args.poly_num as Angle);
                    let poly_point = point + out_vec * angle.sin() * self.method_args.radius - up_vec * angle.cos() * self.method_args.radius;
                    corner_slice.push(poly_point);
                }

                corner_slices.push(corner_slice);
            }

            // For each corner, mesh the section to the next corner
            for (slice_id, coil_vertex) in coil.vertices.iter().enumerate() {
                let next_slice_id = coil_vertex.next_id;
                let slice = &corner_slices[slice_id];
                let next_slice = &corner_slices[next_slice_id];

                if slice.len() != next_slice.len() {
                    mesh::err_str(&format!("BUG: Coil corner {0} has a different number of points ({1}) than the next {2} ({3})", 
                        slice_id, slice.len(), next_slice_id, next_slice.len()))?;
                }
                
                for (i, v0) in slice.iter().enumerate() {
                    let i_next = (i + 1) % slice.len();
                    let v1 = &slice[i_next];
                    let w0 = &next_slice[i];
                    let w1 = &next_slice[i_next];

                    let n0 = (v1 - v0).cross(&(w0 - v0)).normalize();
                    let n1 = (v1 - w0).cross(&(w1 - w0)).normalize();

                    triangles.push(stl_triangle(&n0, v0, v1, w0));
                    triangles.push(stl_triangle(&n1, v1, w1, w0));

                    full_triangles.push(stl_triangle(&n0, v0, v1, w0));
                    full_triangles.push(stl_triangle(&n1, v1, w1, w0));
                }
            }

            // Save each coil to a separate file
            let numbered_output_path = output_path.replace(".stl", &format!("_c{}.stl", coil_n));
            println!("Saving coil {} to {}...", coil_n, numbered_output_path);
            save_stl(&triangles, &numbered_output_path)?;
        }

        // Save a full set of coils (often just for visualization)
        println!("Saving full array to {}", output_path);
        save_stl(&full_triangles, output_path)?;

        Ok(())
    }
}

fn save_stl(triangles: &Vec<stl_io::Triangle>, output_path: &str) -> mesh::ProcResult<()> {
    let mut file = match OpenOptions::new().write(true).create(true).open(&output_path)
    {
        Ok(file) => file,
        Err(error) => {
            return Err(crate::io::IoError{file: output_path.to_string(), cause: error}.into());
        },
    };
    match stl_io::write_stl(&mut file, triangles.iter())
    {
        Ok(_) => (),
        Err(error) => {
            return Err(crate::io::IoError{file: output_path.to_string(), cause: error}.into());
        },
    };
    Ok(())
}

/// Helper function for triangle construction.
fn stl_triangle(normal: &GeoVector, v0: &Point, v1: &Point, v2: &Point) -> stl_io::Triangle {
    stl_io::Triangle{
        normal: stl_io::Normal::new([normal.x, normal.y, normal.z]),
        vertices: [
            stl_io::Vertex::new([v0.x, v0.y, v0.z]),
            stl_io::Vertex::new([v1.x, v1.y, v1.z]),
            stl_io::Vertex::new([v2.x, v2.y, v2.z]),
        ]
    }
} 
