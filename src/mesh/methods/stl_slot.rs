use crate::{
    layout,
    mesh,
    io,
};
use mesh::methods;
use crate::geo_3d::*;

use serde::{Serialize, Deserialize};
use std::f32::consts::PI;

/// STL Slot Method struct.
/// This struct contains all the parameters for the STL Slot meshing method.
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Method {
    #[serde(default = "Method::default_radius_offset", alias = "offset")]
    radius_offset: f32,
    #[serde(default = "Method::default_poly_num")]
    poly_num: usize,
    #[serde(default = "Method::default_slot_depth")]
    slot_depth: f32,
    #[serde(default = "Method::default_save_individual", alias = "individual")]
    save_individual: bool,
}
impl Method {
    pub fn default_radius_offset() -> f32 {
        0.0
    }
    pub fn default_poly_num() -> usize {
        8
    }
    pub fn default_slot_depth() -> f32 {
        10.0
    }
    pub fn default_save_individual() -> bool {
        false
    }
}
impl Default for Method {
    fn default() -> Self {
        Method{
            radius_offset: Method::default_radius_offset(),
            poly_num: Method::default_poly_num(),
            slot_depth: Method::default_slot_depth(),
            save_individual: Method::default_save_individual(),
        }
    }
}

impl methods::MeshMethodTrait for Method {
    /// Get the name of the meshing method.
    fn get_method_display_name(&self) -> &'static str {
        "STL Slot"
    }

    /// Get the output file extension for the meshing method.
    fn get_output_extension(&self) -> &'static str {
        "stl"
    }

    /// Run the meshing process with the given arguments.
    /// Uses the `mesh` and `layout` modules.
    fn save_mesh(&self, layout: &layout::Layout, output_path: &str) -> mesh::ProcResult<()> {
        let output_path = output_path.to_string() + ".stl";

        let mut full_triangles = Vec::<stl_io::Triangle>::new();

        // Mesh each coil
        for (coil_n, coil) in layout.coils.iter().enumerate() {

            let radius = coil.wire_radius + self.radius_offset;

            // Initialize the triangle list
            let mut triangles = Vec::<stl_io::Triangle>::new();

            // Create the corner slice polygons
            let mut corner_slices = Vec::<Vec::<Point>>::new();

            let bottom_poly_count = (self.poly_num as f32 / 2.0).ceil() as usize;
            let is_even = self.poly_num % 2 == 0;

            for coil_vertex in coil.vertices.iter() {
                let mut corner_slice = Vec::new();

                let point = coil_vertex.point;

                let up_vec = coil_vertex.wire_radius_normal.normalize();
                let out_vec = (point - coil.center).rej_onto(&up_vec).normalize();

                let slot_up_vec = coil_vertex.surface_normal.normalize() * self.slot_depth;

                // Put the polygon points around the plane given by the point and the out_vec/up_vec
                for i in 0..bottom_poly_count{
                    let angle = 2.0 * PI * (i as Angle + 0.5) / (self.poly_num as Angle) - PI;
                    let poly_point = point + (out_vec * angle.cos() + up_vec * angle.sin()) * radius;
                    corner_slice.push(poly_point);
                }
                // Add the top points
                if !is_even {
                    let poly_point = point + out_vec * radius;
                    corner_slice.push(poly_point);
                }
                corner_slice.push(point + slot_up_vec + out_vec * radius);
                corner_slice.push(point + slot_up_vec - out_vec * radius);
                if !is_even {
                    let poly_point = point - out_vec * radius;
                    corner_slice.push(poly_point);
                }

                corner_slices.push(corner_slice);
            }

            // For each corner, mesh the section to the next corner
            for slice_id in 0..coil.vertices.len() {
                let next_slice_id = (slice_id + 1) % corner_slices.len();
                let slice = &corner_slices[slice_id];
                let next_slice = &corner_slices[next_slice_id];

                if slice.len() != next_slice.len() {
                    panic!("BUG: Coil corner {0} has a different number of points ({1}) than the next {2} ({3})", 
                        slice_id, slice.len(), next_slice_id, next_slice.len());
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
            if self.save_individual {
                // Save each coil to a separate file
                let numbered_output_path = output_path.replace(".stl", &format!("_c{}.stl", coil_n));
                io::stl::save_stl_from_triangles(&triangles, &numbered_output_path)?;
            }
        }

        // Save a full set of coils (often just for visualization)
        println!("Saving full array to {}", output_path);
        io::stl::save_stl_from_triangles(&full_triangles, &output_path)?;

        Ok(())
    }
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
