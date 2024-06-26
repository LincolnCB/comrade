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
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
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
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
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

            coil_vertices.push(CoilVertex{
                point: point.clone(),
                surface_normal: point_normals[point_id].clone(),
                wire_radius_normal: point_normals[point_id].clone(),
            });
        }

        Ok(Coil{center, normal, wire_radius, vertices: coil_vertices, port: None, breaks: Vec::new()})
    }

    /// Calculate the wire length of the coil, in mm
    pub fn wire_length(&self) -> f32 {
        let mut length = 0.0;
        for (id, vertex) in self.vertices.iter().enumerate() {
            length += vertex.point.distance(&self.vertices[(id + 1) % self.vertices.len()].point);
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

    /// Calculate the mutual inductance between two coils, as well as the gradient
    /// with respect to the x, y, and z coordinates of the first coil.
    /// Returns a tuple of (M [nH], dMx [nH/mm], dMy [nH/mm], dMz [nH/mm]).
    /// The gradient with respect to the second coil position will be the negative of gradient returned here.
    /// dl is the maximum length infinitessimal approximation within a segment.
    /// For example, for a wire segment of length 2.3 * dl,
    /// there will be two segments of length dl and one of length 0.3 * dl.
    /// This value will have no effect on the calculation if longer than a given segment length.
    pub fn mutual_inductance_info(&self, other: &Coil, dl: f32, calc_val: bool, calc_dxyz: bool, calc_dr: bool) -> (Option<f32>, Option<f32>, Option<f32>, Option<f32>, Option<f32>) {
        let d_thresh = 0.25; // Threshold for distance between points

        let mut lambda = 0.0;
        let mut lambda_dx = 0.0;
        let mut lambda_dy = 0.0;
        let mut lambda_dz = 0.0;
        let mut lambda_dr = 0.0;
        // dl * dl is reused often, so calculate it once
        let dl_sq = dl * dl;

        for (id, vertex) in self.vertices.iter().enumerate() {
            // Lay out the first coil segment
            let p0 = vertex.point;
            let p1 = self.vertices[(id + 1) % self.vertices.len()].point;
            let np = (p1 - p0).normalize();
            let dp = p0.distance(&p1);
            let i_max = (dp / dl).floor() as u32;
            let dp_remainder = dp - (i_max as f32) * dl;
            let dp_remainder_normalized = dp_remainder / dp;

            let mut update = |p: Point, q: Point, scale: f32| {
                if calc_val {
                    lambda += scale / p.distance(&q);
                }
                if calc_dxyz || calc_dr {
                    let dist_cub = p.distance(&q).powi(3);
                    let d_scale = scale / dist_cub;
                    let dx = d_scale * (q.x - p.x);
                    let dy = d_scale * (q.y - p.y);
                    let dz = d_scale * (q.z - p.z);
                    if calc_dxyz {
                        lambda_dx += dx;
                        lambda_dy += dy;
                        lambda_dz += dz;
                    }
                    if calc_dr {
                        lambda_dr += GeoVector::new(dx, dy, dz).proj_onto(&(p-self.center)).norm();
                    }
                }
            };

            for (other_id, other_vertex) in other.vertices.iter().enumerate() {
                // Lay out the second coil segment
                let q0 = other_vertex.point;
                let q1 = other.vertices[(other_id + 1) % other.vertices.len()].point;
                let nq = (q1 - q0).normalize();
                let dq = q0.distance(&q1);
                let j_max = (dq / dl).floor() as u32;
                let dq_remainder = dq - (j_max as f32) * dl;
                let dq_remainder_normalized = dq_remainder / dq;

                // Get the dot product of the two normalized segments
                let dot = np.dot(&nq);
                // dl * dl * dot is reused often, so calculate it once
                let dl_sq_dot = dl_sq * dot;

                // Iterate over sub-segments
                for i in 0..i_max {
                    let p = p0 + np * (i as f32 + 0.5) * dl;
                    for j in 0..j_max {
                        let q = q0 + nq * (j as f32 + 0.5) * dl;
                        if p.distance(&q) > d_thresh * (self.wire_radius + other.wire_radius) {
                            update(p, q, dl_sq_dot);
                        }
                    }
                    // Remainder for second segment
                    let q = q0 + nq * (1.0 - 0.5 * dq_remainder_normalized) * dq;
                    if p.distance(&q) > d_thresh * (self.wire_radius + other.wire_radius) {
                        update(p, q, dl * dq_remainder * dot);
                    }
                }
                // Remainder for first segment
                let p = p0 + np * (1.0 - 0.5 * dp_remainder_normalized) * dp;
                for j in 0..j_max {
                    let q = q0 + nq * (j as f32 + 0.5) * dl;
                    if p.distance(&q) > d_thresh * (self.wire_radius + other.wire_radius) {
                        update(p, q, dp_remainder * dl * dot);
                    }
                }
                // Remainder for both segments
                let q = q0 + nq * (1.0 - 0.5 * dq_remainder_normalized);
                if p.distance(&q) > d_thresh * (self.wire_radius + other.wire_radius) {
                    update(p, q, dp_remainder * dq_remainder * dot);
                }
            }
        }
        // Multiply by the constant factor of mu0/4pi. mu0 is already in units of nH/mm.
        let out = |l, calc| -> Option<f32> {
            if calc {
                Some(MU0 * l / (4.0 * PI))
            } else {
                None
            }
        };

        (out(lambda, calc_val), out(lambda_dx, calc_dxyz), out(lambda_dy, calc_dxyz), out(lambda_dz, calc_dxyz), out(lambda_dr, calc_dr))
    }

    /// Wrapper to calculate the mutual inductance between two coils, in nH.
    pub fn mutual_inductance(&self, other: &Coil, dl: f32) -> f32 {
        let (m, _, _, _, _) = self.mutual_inductance_info(other, dl, true, false, false);
        m.unwrap()
    }

    /// Wrapper to calculate the mutual inductance between two coils, as well as the gradient wrt only the radius
    pub fn mutual_inductance_dradius(&self, other: &Coil, dl: f32) -> (f32, f32) {
        let (m, _, _, _, dr) = self.mutual_inductance_info(other, dl, true, false, true);
        (m.unwrap(), dr.unwrap())
    }

    /// Wrapper to calculate the full info of the mutual inductance between two coils.
    pub fn mutual_inductance_full(&self, other: &Coil, dl: f32) -> (f32, f32, f32, f32, f32) {
        let (m, dx, dy, dz, dr) = self.mutual_inductance_info(other, dl, true, true, true);
        (m.unwrap(), dx.unwrap(), dy.unwrap(), dz.unwrap(), dr.unwrap())
    }

    /// Calculate the coupling factor between two coils.
    pub fn coupling_factor(&self, other: &Coil, dl: f32) -> f32 {
        self.mutual_inductance(other, dl) / (self.self_inductance(dl) * other.self_inductance(dl)).sqrt()
    }
}

/// A point on a coil (includes adjacency and surface vectors).
#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
pub struct CoilVertex {
    pub point: Point,
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
    println!();
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
