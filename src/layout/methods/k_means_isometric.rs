/*!
*   Alternating Circles Method
*
*
!*/

use crate::layout;
use crate::geo_3d::*;
use layout::methods;
use methods::alternating_circles::Method as AlternatingCirclesMethod;
use methods::alternating_circles::CircleArgs as Circle;
use methods::helper::{
    k_means,
    closest_point,
};

use serde::{Serialize, Deserialize};

/// Alternating Circles Method struct.
/// This struct contains all the parameters for the Alternating Circles layout method.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Method {
    // Circle intersection parameters
    #[serde(default = "Method::default_circles")]
    circles: usize,
    #[serde(default = "Method::default_epsilon")]
    epsilon: f32,
    #[serde(default = "Method::default_pre_shift")]
    pre_shift: bool,

    // Overlap handling parameters
    #[serde(default = "Method::default_clearance")]
    clearance: f32,
    #[serde(default = "Method::default_wire_radius")]
    wire_radius: f32,
    #[serde(default = "Method::default_zero_angle_vector")]
    zero_angle_vector: GeoVector,
    #[serde(default = "Method::default_backup_zero_angle_vector")]
    backup_zero_angle_vector: GeoVector,

    // Iteration parameters
    #[serde(default = "Method::default_iterations")]
    iterations: usize,
    #[serde(default = "Method::default_initial_step")]
    initial_step: f32,
    #[serde(default = "Method::default_step_decrease")]
    step_decrease: f32,
    #[serde(default = "Method::default_radius_freedom")]
    radius_freedom: f32,
    #[serde(default = "Method::default_center_freedom")]
    center_freedom: f32,
    #[serde(default = "Method::default_close_cutoff")]
    close_cutoff: f32,
    #[serde(default = "Method::default_radial_stiffness", alias = "stiffness")]
    radial_stiffness: f32,

    // Verbosity
    #[serde(default = "Method::default_verbose")]
    verbose: bool,

    // Save final cfg output
    #[serde(default = "Method::default_final_cfg_output")]
    final_cfg_output: Option<String>,
}
impl Method {
    pub fn default_circles() -> usize {
        12
    }
    pub fn default_epsilon() -> f32 {
        0.15
    }
    pub fn default_pre_shift() -> bool {
        true
    }

    pub fn default_clearance() -> f32 {
        1.29
    }
    pub fn default_wire_radius() -> f32 {
        0.645
    }
    pub fn default_zero_angle_vector() -> GeoVector {
        GeoVector::zhat()
    }
    pub fn default_backup_zero_angle_vector() -> GeoVector {
        GeoVector::yhat()
    }

    pub fn default_iterations() -> usize {
        1
    }
    pub fn default_initial_step() -> f32 {
        1.0
    }
    pub fn default_step_decrease() -> f32 {
        0.5
    }
    pub fn default_center_freedom() -> f32 {
        0.5
    }
    pub fn default_radius_freedom() -> f32 {
        0.15
    }
    pub fn default_close_cutoff() -> f32 {
        1.1
    }
    pub fn default_radial_stiffness() -> f32 {
        1.0
    }

    pub fn default_verbose() -> bool {
        false
    }
    pub fn default_final_cfg_output() -> Option<String> {
        None
    }
}
impl Default for Method{
    fn default() -> Self {
        Method{
            circles: Self::default_circles(),
            epsilon: Self::default_epsilon(),
            pre_shift: Self::default_pre_shift(),

            clearance: Self::default_clearance(),
            wire_radius: Self::default_wire_radius(),
            zero_angle_vector: Self::default_zero_angle_vector(),
            backup_zero_angle_vector: Self::default_backup_zero_angle_vector(),

            iterations: Self::default_iterations(),
            initial_step: Self::default_initial_step(),
            step_decrease: Self::default_step_decrease(),
            center_freedom: Self::default_center_freedom(),
            radius_freedom: Self::default_radius_freedom(),
            close_cutoff: Self::default_close_cutoff(),
            radial_stiffness: Self::default_radial_stiffness(),

            verbose: Self::default_verbose(),
            final_cfg_output: Self::default_final_cfg_output(),
        }
    }

}

impl methods::LayoutMethodTrait for Method {
    /// Get the name of the layout method.
    fn get_method_display_name(&self) -> &'static str {
        "K-means Isometric Circles"
    }

    fn do_layout(&self, surface: &Surface) -> layout::ProcResult<layout::Layout> {

        let mut centers = Vec::<Point>::new();
        let mut radius = 5.0;
        let boundary_points = surface.get_boundary_vertex_indices().iter().map(|v| surface.vertices[*v].point).collect();

        // Iteratively trim the boundary until the centers are a sufficient distance from the boundary
        let mut temp_points = surface.vertices.iter().map(|v| v.point).collect::<Vec<Point>>();
        let mut boundary_trim = 0.0;

        if self.verbose {
            println!("Selecting centers...");
        }
        for _ in 0..5 {
            // Generate centers
            centers = k_means(&temp_points, self.circles, 1000, false);

            // Calculate radius
            radius = 0.0;
            let mut boundary_dist = 0.0;
            let mut centers_near_boundary = 0;
            for i in 0..centers.len(){

                let mut min_dist = std::f32::MAX;
                for j in 0..centers.len(){
                    if i != j {
                        let dist = centers[i].distance(&centers[j]);
                        if dist < min_dist {
                            min_dist = dist;
                        }
                    }
                }

                // Track distance to boundary for centers closer to the boundary than other centers
                let boundary_point = *closest_point(&centers[i], &boundary_points);
                if boundary_point.distance(&centers[i]) - boundary_trim < min_dist {
                    boundary_dist += boundary_point.distance(&centers[i]);
                    centers_near_boundary += 1;
                }

                // Calculate the average distance to nearby centers
                let mut avg_nearby_dist = 0.0;
                let mut nearby_count = 0;
                for j in 0..centers.len(){
                    if i != j {
                        let dist = centers[i].distance(&centers[j]);
                        if dist < 1.35 * min_dist {
                            avg_nearby_dist += dist;
                            nearby_count += 1;
                        }
                    }
                }

                radius += avg_nearby_dist / nearby_count as f32;
            }
            radius /= 1.5 * centers.len() as f32;
            boundary_dist /= centers_near_boundary as f32;

            if boundary_dist < radius {
                println!("Trimming boundary...");
                boundary_trim += 0.9 * (radius - boundary_dist);

                // Trim the points
                temp_points = temp_points.iter().filter(|p| {
                    let mut keep = true;
                    for b in boundary_points.iter() {
                        if p.distance(b) < boundary_trim {
                            keep = false;
                            break;
                        }
                    }
                    keep
                }).map(|p| *p).collect::<Vec<Point>>();
            } else {
                break;
            }
        }

        // Map to circles
        let circles = centers.iter().map(|c| Circle{
                center: *c, 
                coil_radius: radius, 
                break_count: Circle::default_break_count(),
                break_angle_offset: Circle::default_break_angle_offset(),
            }).collect();

        // Create method
        let method = AlternatingCirclesMethod{
            circles,
            epsilon: self.epsilon,
            pre_shift: self.pre_shift,

            clearance: self.clearance,
            wire_radius: self.wire_radius,
            zero_angle_vector: self.zero_angle_vector,
            backup_zero_angle_vector: self.backup_zero_angle_vector,

            iterations: self.iterations,
            initial_step: self.initial_step,
            step_decrease: self.step_decrease,
            center_freedom: self.center_freedom,
            radius_freedom: self.radius_freedom,
            close_cutoff: self.close_cutoff,
            radial_stiffness: self.radial_stiffness,

            verbose: self.verbose,
            warn_on_shift: false,
            final_cfg_output: self.final_cfg_output.clone(),
        };

        // Run method
        method.do_layout(surface)
    }
}
