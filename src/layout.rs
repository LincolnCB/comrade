mod proc_errors;
mod cfg;
mod methods;

use serde::{Serialize, Deserialize};

use std::f32::consts::PI;
const MU0: f32 = 1.256637062; // mu0 in nH/mm

use crate::geo_3d::*;

// Re-export errors
pub use proc_errors::{
    LayoutError,
    ProcResult,
    err_str,
};
// Re-export cfg handling
pub use cfg::LayoutTarget;

// Re-export layout methods
pub use methods::{
    MethodEnum,
    LayoutMethodTrait,
};

/// Layout struct.
/// This struct contains all the necessary results from the layout process.
/// Returned from the layout process, used as input to the matching process.
#[derive(Debug, Serialize, Deserialize)]
pub struct Layout {
    pub coils: Vec<Coil>,
}
impl Layout {
    /// Create a new layout.
    pub fn new() -> Self{
        Layout{coils: Vec::new()}
    }
}

/// A coil.
/// Contains a list of points.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Coil {
    pub center: Point,
    pub normal: GeoVector,
    pub wire_radius: f32,
    pub vertices: Vec<CoilVertex>,
    pub port: Option<usize>,
    pub breaks: Vec<usize>,
}
impl Coil {
    /// Create a new coil.
    /// Points must be in order -- the coil will be closed automatically.
    pub fn new(
        center: Point,
        normal: GeoVector,
        points: Vec<Point>,
        wire_radius: f32,
        point_normals: Vec<GeoVector>,
    ) -> ProcResult<Self>{

        // Check that there are at least 3 points
        if points.len() < 3 {
            err_str("Coil must have at least 3 points!")?;
        }

        // Check that the number of points and normals match
        if points.len() != point_normals.len() {
            err_str("Number of points and normals must match!")?;
        }

        // Connect the points
        let mut coil_vertices = Vec::<CoilVertex>::new();

        for (point_id, point) in points.iter().enumerate() {
            let next_id = (point_id + 1) % points.len();
            let prev_id = (point_id + points.len() - 1) % points.len();

            coil_vertices.push(CoilVertex{
                point: point.clone(),
                id: point_id,
                next_id,
                prev_id,
                surface_normal: point_normals[point_id].clone(),
                wire_radius_normal: point_normals[point_id].clone(),
            });
        }

        Ok(Coil{center, normal, wire_radius, vertices: coil_vertices, port: None, breaks: Vec::new()})
    }

    /// Calculate the wire length of the coil, in mm
    pub fn wire_length(&self) -> f32 {
        let mut length = 0.0;
        for vertex in self.vertices.iter() {
            length += vertex.point.distance(&self.vertices[vertex.next_id].point);
        }
        length
    }

    /// Calculate the average radius of the coil, in mm
    pub fn average_radius(&self) -> f32 {
        let mut radius = 0.0;
        for vertex in self.vertices.iter() {
            radius += vertex.point.distance(&self.center);
        }
        radius / (self.vertices.len() as f32)
    }

    /// Calculate the self-inductance of the coil, in nH.
    pub fn self_inductance(&self, dl:f32) -> f32 {
        // TODO: This may depend on frequency, so it may need to be updated.
        const CORRECTION_SCALE : f32 = 0.0012;
        let correction_factor = CORRECTION_SCALE * self.average_radius() / self.wire_radius;
        self.mutual_inductance(&self, dl) + self.wire_length() * correction_factor
    }

    /// Calculate the mutual inductance between two coils, in nH.
    /// dl is the maximum length infinitessimal approximation within a segment.
    /// For example, for a wire segment of length 2.3 * dl,
    /// there will be two segments of length dl and one of length 0.3 * dl.
    /// This value will have no effect on the calculation if longer than a given segment length.
    pub fn mutual_inductance(&self, other: &Coil, dl: f32) -> f32 {
        let d_thresh = 0.25; // Threshold for distance between points

        let mut lambda = 0.0;
        let dl_sq = dl * dl;

        for vertex in self.vertices.iter() {
            // Lay out the first coil segment
            let p0 = vertex.point;
            let p1 = self.vertices[vertex.next_id].point;
            let np = (p1 - p0).normalize();
            let dp = p0.distance(&p1);
            let i_max = (dp / dl).floor() as u32;
            let dp_remainder = dp - (i_max as f32) * dl;
            let dp_remainder_normalized = dp_remainder / dp;

            for other_vertex in other.vertices.iter() {
                // Lay out the second coil segment
                let q0 = other_vertex.point;
                let q1 = other.vertices[other_vertex.next_id].point;
                let nq = (q1 - q0).normalize();
                let dq = q0.distance(&q1);
                let j_max = (dq / dl).floor() as u32;
                let dq_remainder = dq - (j_max as f32) * dl;
                let dq_remainder_normalized = dq_remainder / dq;

                // Get the dot product of the two normalized segments
                let dot = np.dot(&nq);
                let dl_sq_dot = dl_sq * dot;

                // Iterate over sub-segments
                for i in 0..i_max {
                    let p = p0 + np * (i as f32 + 0.5) * dl;
                    for j in 0..j_max {
                        let q = q0 + nq * (j as f32 + 0.5) * dl;
                        if p.distance(&q) > d_thresh * (self.wire_radius + other.wire_radius) {
                            lambda += dl_sq_dot / p.distance(&q)
                        }
                    }
                    // Remainder for second segment
                    let q = q0 + nq * (1.0 - 0.5 * dq_remainder_normalized) * dq;
                    if p.distance(&q) > d_thresh * (self.wire_radius + other.wire_radius) {
                        lambda += dl * dq_remainder * dot / p.distance(&q);
                    }
                }
                // Remainder for first segment
                let p = p0 + np * (1.0 - 0.5 * dp_remainder_normalized) * dp;
                for j in 0..j_max {
                    let q = q0 + nq * (j as f32 + 0.5) * dl;
                    if p.distance(&q) > d_thresh * (self.wire_radius + other.wire_radius) {
                        lambda += dl * dp_remainder * dot / p.distance(&q);
                    }
                }
                // Remainder for both segments
                let q = q0 + nq * (1.0 - 0.5 * dq_remainder_normalized);
                if p.distance(&q) > d_thresh * (self.wire_radius + other.wire_radius) {
                    lambda += dp_remainder * dq_remainder * dot / p.distance(&q);
                }
            }
        }
        // Multiply by the constant factor of mu0/4pi. mu0 is already in units of nH/mm.
        MU0 * lambda / (4.0 * PI)
    }

    /// Calculate the coupling factor between two coils.
    pub fn coupling_factor(&self, other: &Coil, dl: f32) -> f32 {
        self.mutual_inductance(other, dl) / (self.self_inductance(dl) * other.self_inductance(dl)).sqrt()
    }
}

/// A point on a coil (includes adjacency and surface vectors).
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoilVertex {
    pub point: Point,
    pub id: usize,
    pub next_id: usize,
    pub prev_id: usize,
    pub surface_normal: GeoVector,
    pub wire_radius_normal: GeoVector,
}

/// Run the layout process.
/// Returns a `ProcResult` with the `Layout` or an `Err`.
pub fn do_layout(layout_target: &LayoutTarget) -> ProcResult<Layout> {
    // Extract the layout method
    let layout_method = &layout_target.method;

    // Load the input
    let surface = layout_method.load_surface(&layout_target.input_path)?;

    // Run the layout method
    println!("Running layout method: {}...", layout_method.get_method_display_name());
    layout_method.do_layout(&surface)
}

pub fn save_layout(layout: &Layout, output_path: &str) -> ProcResult<()> {
    assert!(output_path.ends_with(".json"), "Output path must end with .json -- cfg file loader should check this!");
    crate::io::save_ser_to(output_path, layout)?;
    Ok(())
}

pub fn load_layout(input_path: &str) -> ProcResult<Layout> {
    assert!(input_path.ends_with(".json"), "Input path must end with .json -- cfg file loader should check this!");
    let layout: Layout = crate::io::load_deser_from(input_path)?;
    Ok(layout)
}
