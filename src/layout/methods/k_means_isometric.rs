/*!
*   Alternating Circles Method
*
*
!*/

use crate::layout;
use crate::geo_3d::*;
use layout::methods;
use methods::adam_circles::Method as AdamCirclesMethod;
use methods::adam_circles::CircleArgs as Circle;
use methods::helper::{
    k_means,
    k_means_initialized,
    closest_point,
};

use serde::{Serialize, Deserialize};

/// K Means Isometric Circles Method
/// Includes the parameters for the Adam Circles method, as well as additional parameters for the k-means algorithm.
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Method {
    // K-means parameters
    // --------------------------------
    #[serde(default = "Method::default_circles")]
    circles: usize,
    #[serde(default = "Method::default_symmetry_plane", alias = "plane")]
    symmetry_plane: Option<Plane>,
    #[serde(default = "Method::default_initial_centers")]
    initial_centers: Option<Vec<Point>>,
    
    // Visualization (no optimization, just display the centers as small loops)
    #[serde(default = "Method::default_visualize")]
    visualize: bool,
    
    // Save final centers output
    #[serde(default = "Method::default_centers_output")]
    centers_output: Option<String>,
    // --------------------------------

    // Optimization parameters
    // --------------------------------
    // Circle intersection parameters
    #[serde(default = "Method::default_epsilon")]
    pub epsilon: f32,
    #[serde(default = "Method::default_pre_shift")]
    pub pre_shift: bool,

    // Overlap handling parameters
    #[serde(default = "Method::default_clearance")]
    pub clearance: f32,
    #[serde(default = "Method::default_wire_radius")]
    pub wire_radius: f32,
    #[serde(default = "Method::default_zero_angle_vector")]
    pub zero_angle_vector: GeoVector,
    #[serde(default = "Method::default_backup_zero_angle_vector")]
    pub backup_zero_angle_vector: GeoVector,

    // Iteration parameters
    #[serde(default = "Method::default_iterations")]
    pub iterations: usize,
    #[serde(default = "Method::default_step_size")]
    pub step_size: f32,
    #[serde(default = "Method::default_first_moment_decay", alias = "b1")]
    pub first_moment_decay: f32,
    #[serde(default = "Method::default_second_moment_decay", alias = "b2")]
    pub second_moment_decay: f32,
    #[serde(default = "Method::default_radius_reg", alias = "radius_regularization")]
    pub radius_reg: f32,
    #[serde(default = "Method::default_radius_freedom")]
    pub radius_freedom: f32,
    #[serde(default = "Method::default_center_freedom")]
    pub center_freedom: f32,
    #[serde(default = "Method::default_close_cutoff")]
    pub close_cutoff: f32,

    // Verbosity
    #[serde(default = "Method::default_verbose")]
    pub verbose: bool,
    #[serde(default = "Method::default_warn_on_shift")]
    pub warn_on_shift: bool,
    #[serde(default = "Method::default_statistics_level", alias = "statistics")]
    pub statistics_level: u32,

    // Save final cfg output
    #[serde(default = "Method::default_final_cfg_output")]
    pub final_cfg_output: Option<String>,
}
impl Method {
    pub fn default_circles() -> usize {
        12
    }
    pub fn example_symmetry_plane() -> Option<Plane> {
        Some(Plane::from_normal_and_offset(GeoVector::xhat(), 0.0))
    }
    pub fn default_symmetry_plane() -> Option<Plane> {
        None
    }
    pub fn example_initial_centers() -> Option<Vec<Point>> {
        Some(vec![
            Point::new(0.0, 0.0, 0.0),
            Point::new(1.0, 0.0, 0.0),
            Point::new(0.0, 1.0, 0.0),
            Point::new(1.0, 1.0, 0.0),
        ])
    }
    pub fn default_initial_centers() -> Option<Vec<Point>> {
        None
    }
    pub fn default_visualize() -> bool {
        false
    }
    pub fn example_centers_output() -> Option<String> {
        Some("PATH/TO/OUTPUT/centers.[json|yaml|toml]".to_string())
    }
    pub fn default_centers_output() -> Option<String> {
        None
    }
    
    pub fn default_epsilon() -> f32 {
        1.5
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
        0
    }
    pub fn example_iterations() -> usize {
        5
    }
    pub fn default_step_size() -> f32 {
        0.2
    }
    pub fn default_first_moment_decay() -> f32 {
        0.9
    }
    pub fn default_second_moment_decay() -> f32 {
        0.999
    }
    pub fn default_center_freedom() -> f32 {
        0.95
    }
    pub fn default_radius_freedom() -> f32 {
        0.65
    }
    pub fn default_close_cutoff() -> f32 {
        0.95
    }
    pub fn default_radius_reg() -> f32 {
        0.1
    }

    pub fn default_verbose() -> bool {
        false
    }
    pub fn default_warn_on_shift() -> bool {
        true
    }
    pub fn default_statistics_level() -> u32 {
        0
    }

    pub fn example_final_cfg_output() -> Option<String> {
        Some("PATH/TO/FINAL/CFG.[json|yaml|toml]".to_string())
    }
    pub fn default_final_cfg_output() -> Option<String> {
        None
    }
}
impl Default for Method{
    fn default() -> Self {
        Method{
            circles: Self::default_circles(),
            symmetry_plane: Self::example_symmetry_plane(),
            initial_centers: Self::example_initial_centers(),
            visualize: Self::default_visualize(),
            centers_output: Self::example_centers_output(),

            epsilon: Self::default_epsilon(),
            pre_shift: Self::default_pre_shift(),

            clearance: Self::default_clearance(),
            wire_radius: Self::default_wire_radius(),
            zero_angle_vector: Self::default_zero_angle_vector(),
            backup_zero_angle_vector: Self::default_backup_zero_angle_vector(),

            iterations: Self::example_iterations(),
            step_size: Self::default_step_size(),
            first_moment_decay: Self::default_first_moment_decay(),
            second_moment_decay: Self::default_second_moment_decay(),
            center_freedom: Self::default_center_freedom(),
            radius_freedom: Self::default_radius_freedom(),
            close_cutoff: Self::default_close_cutoff(),
            radius_reg: Self::default_radius_reg(),

            verbose: Self::default_verbose(),
            warn_on_shift: Self::default_warn_on_shift(),
            statistics_level: Self::default_statistics_level(),

            final_cfg_output: Self::example_final_cfg_output(),
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

        for it in 0..5 {

            if self.verbose {
                println!("Trim pass: {}/5", it + 1);
                println!();
            }

            // Generate centers
            let initial_centers = if let Some(initial_centers) = &self.initial_centers {
                Some(initial_centers.clone())
            } else if it > 0 {
                Some(centers.clone())
            } else {
                None
            };
            if self.symmetry_plane.is_none() {
                centers = self.k_means(&temp_points, &initial_centers, 1000);
            } else {
                centers = self.k_means_symmetric(&temp_points, &initial_centers, 1000);
            }

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
                println!();
                boundary_trim += 1.1 * (radius - boundary_dist);

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

        // Just display the centers if visualize
        radius = if self.visualize { 5.0 } else { radius };

        // Save centers if requested
        if let Some(output_path) = &self.centers_output {
            crate::io::save_ser_to(output_path, &centers)?;
        }


        // Map to circles
        let circles = if let Some(symmetry_plane) = &self.symmetry_plane {
            centers.iter()
            .filter(|c| symmetry_plane.distance_to_point(c) >= -1e-6)
            .map(|c| Circle{
                center: *c, 
                coil_radius: radius, 
                break_count: Circle::default_break_count(),
                break_angle_offset: Circle::default_break_angle_offset(),
                on_symmetry_plane: symmetry_plane.distance_to_point(c).abs() < 1e-6,
            }).collect()
        } else {
            centers.iter().map(|c| Circle{
                center: *c, 
                coil_radius: radius, 
                break_count: Circle::default_break_count(),
                break_angle_offset: Circle::default_break_angle_offset(),
                on_symmetry_plane: false,
            }).collect()
        };

        let iterations = if self.visualize { 0 } else { self.iterations };
        if iterations != self.iterations {
            println!("WARNING: Visualization mode enabled, setting iterations to 0.")
        }

        // Create method
        let method = AdamCirclesMethod{
            symmetry_plane: self.symmetry_plane,
            layout_in_path: None,

            circles,
            epsilon: self.epsilon,
            pre_shift: self.pre_shift,

            clearance: self.clearance,
            wire_radius: self.wire_radius,
            zero_angle_vector: self.zero_angle_vector,
            backup_zero_angle_vector: self.backup_zero_angle_vector,

            iterations: self.iterations,
            step_size: self.step_size,
            first_moment_decay: self.first_moment_decay,
            second_moment_decay: self.second_moment_decay,
            center_freedom: self.center_freedom,
            radius_freedom: self.radius_freedom,
            close_cutoff: self.close_cutoff,
            radius_reg: self.radius_reg,

            verbose: self.verbose,
            warn_on_shift: self.warn_on_shift,
            statistics_level: self.statistics_level,

            final_cfg_output: self.final_cfg_output.clone(),
        };

        // Run method
        method.do_layout(surface)
    }
}

impl Method {
    fn k_means(&self, points: &Vec<Point>, initial_centers_option: &Option<Vec<Point>>, max_iter: usize) -> Vec<Point> {
        if let Some(initial_centers) = initial_centers_option.as_ref() {
            k_means_initialized(points, initial_centers, max_iter, false)
        } else {
            k_means(points, self.circles, max_iter, false)
        }
    }

    fn k_means_symmetric(&self, points: &Vec<Point>, initial_centers_option: &Option<Vec<Point>>, max_iter: usize)
     -> Vec<Point> {

        assert!(self.symmetry_plane.is_some(), "Symmetry plane must be defined for symmetric k-means.");

        // Initialize the centers
        let centers = if let Some(initial_centers) = initial_centers_option.as_ref() {
            if self.verbose {
                println!("Using initial centers...");
            }
            initial_centers.clone()
        } else {
            if self.verbose {
                println!("Initializing centers...");
            }
            k_means(points, self.circles, max_iter, self.verbose)
        };

        // Symmetrize the centers
        if self.verbose {
            println!("Symmetrizing...");
            println!();
        }
        let mut symmetrized_centers_detailed = self.symmetrize_centers(&centers);

        // Trim asymmetric points
        if self.verbose {
            println!("Start: Trimming asymmetric points...");
        }
        let mut asymmetry = symmetrized_centers_detailed.iter().map(|(_, _, sign)| *sign).sum::<i32>();
        if self.verbose {
            println!("Asymmetry: {}", asymmetry);
            println!("Original count: {}", centers.len());
        }
        let mut symmetrized_centers: Vec<Point> = if asymmetry < 0 {
            symmetrized_centers_detailed.iter().skip(asymmetry.abs() as usize).map(|(p, _, _)| *p).collect()
        } else if asymmetry > 0 {
            symmetrized_centers_detailed.iter().take(centers.len() - (asymmetry.abs() as usize)).map(|(p, _, _)| *p).collect()
        } else {
            symmetrized_centers_detailed.iter().map(|(p, _, _)| *p).collect()
        };
        if self.verbose {
            println!("Final count: {}", symmetrized_centers.len());
            println!();
        }

        if self.verbose {
            println!("Starting k-means iterations...");
        }
        for it in 0..max_iter {
            if self.verbose && (it + 1) % (max_iter as f32 / 10.0) as usize == 0 {
                println!("Iteration: {} / {}", it + 1, max_iter);
            }
            // Single iteration of k-means
            let mut new_centers = k_means_initialized(points, &symmetrized_centers, 1, false);

            // Symmetrize the centers
            symmetrized_centers_detailed = self.symmetrize_centers(&new_centers);
            new_centers = symmetrized_centers_detailed.iter().map(|(p, _, _)| *p).collect();
            
            // Check for convergence
            let mut converged = true;
            for i in 0..symmetrized_centers.len() {
                if symmetrized_centers[i].distance(&new_centers[i]) > 1e-3 {
                    converged = false;
                    break;
                }
            }

            // Update centers
            symmetrized_centers = new_centers;

            if converged {
                break;
            }
        }

        // Final trim
        if self.verbose {
            println!("End: Trimming asymmetric points...");
        }
        asymmetry = symmetrized_centers_detailed.iter().map(|(_, _, sign)| *sign).sum::<i32>();
        if self.verbose {
            println!("Asymmetry: {}", asymmetry);
            println!("Original count: {}", centers.len());
        }
        symmetrized_centers = if asymmetry < 0 {
            symmetrized_centers_detailed.iter().skip(asymmetry.abs() as usize).map(|(p, _, _)| *p).collect()
        } else if asymmetry > 0 {
            symmetrized_centers_detailed.iter().take(centers.len() - (asymmetry.abs() as usize)).map(|(p, _, _)| *p).collect()
        } else {
            symmetrized_centers_detailed.iter().map(|(p, _, _)| *p).collect()
        };
        if self.verbose {
            println!("Final count: {}", symmetrized_centers.len());
            println!();
        }

        symmetrized_centers
    }

    fn symmetrize_centers(&self, centers: &Vec<Point>) -> Vec<(Point, f32, i32)> {
        let symmetry_plane = self.symmetry_plane.expect("Symmetry plane must be defined for symmetric k-means.");

        let sym_centers = centers.iter().map(|c| c.reflect_across(&symmetry_plane)).collect::<Vec<Point>>();
        let mut collected_info = Vec::<(Point, f32, i32)>::new();
        for (id, center) in centers.iter().enumerate() {
            let mut info = (center.clone(), std::f32::MAX, 0);

            let mut min_dist = std::f32::MAX;
            let mut merge_point = *center;
            let mut merge_id = id;
            for (sym_id, sym_center) in sym_centers.iter().enumerate() {
                let dist = center.distance(sym_center);
                if dist < min_dist {
                    min_dist = dist;
                    merge_point = *sym_center;
                    merge_id = sym_id;
                }
            }
            info.0 = *center + (merge_point - *center) / 2.0;
            info.1 = min_dist;

            // Determine the sign of the reflection
            info.2 = if merge_id == id {
                0
            } else if (*center - symmetry_plane).dot(&symmetry_plane.get_normal()) > 0.0 {
                1
            } else {
                -1
            };

            collected_info.push(info);
        }

        // Sort by signed distance
        collected_info.sort_by(|(_, d1, s1), (_, d2, s2)| {
            (d1 * *s1 as f32).partial_cmp(&(d2 * *s2 as f32)).unwrap()
        });

        collected_info
    }

}
