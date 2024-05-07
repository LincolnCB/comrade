/*!
*   Alternating Circles Method
*
*
!*/

use crate::layout;
use crate::geo_3d::*;
use layout::methods;
use methods::helper::{
    sphere_intersect,
    clean_coil_by_angle,
    merge_segments,
    add_even_breaks_by_angle,
    closest_point,
};

use serde::{Serialize, Deserialize};

/// Alternating Circles Method struct.
/// This struct contains all the parameters for the Alternating Circles layout method.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Method {
    // Circle intersection parameters
    pub circles: Vec<CircleArgs>,
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
    #[serde(default = "Method::default_initial_step")]
    pub initial_step: f32,
    #[serde(default = "Method::default_step_decrease")]
    pub step_decrease: f32,
    #[serde(default = "Method::default_radius_freedom")]
    pub radius_freedom: f32,
    #[serde(default = "Method::default_center_freedom")]
    pub center_freedom: f32,
    #[serde(default = "Method::default_close_cutoff")]
    pub close_cutoff: f32,
    #[serde(default = "Method::default_radial_stiffness", alias = "stiffness")]
    pub radial_stiffness: f32,

    // Verbosity
    #[serde(default = "Method::default_verbose")]
    pub verbose: bool,
    #[serde(default = "Method::default_warn_on_shift")]
    pub warn_on_shift: bool,
    #[serde(default = "Method::default_statistics")]
    pub statistics: bool,

    // Save final cfg output
    #[serde(default = "Method::default_final_cfg_output")]
    pub final_cfg_output: Option<String>,
}
impl Method {
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
    pub fn default_warn_on_shift() -> bool {
        true
    }
    pub fn default_statistics() -> bool {
        false
    }

    pub fn default_final_cfg_output() -> Option<String> {
        None
    }
}
impl Default for Method{
    fn default() -> Self {
        Method{
            circles: vec![CircleArgs::default(); 2],
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
            warn_on_shift: Self::default_warn_on_shift(),
            statistics: Self::default_statistics(),

            final_cfg_output: Self::default_final_cfg_output(),
        }
    }

}

/// Single element arguments
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct CircleArgs {
    pub center: Point,
    #[serde(default = "CircleArgs::default_coil_radius", alias = "radius")]
    pub coil_radius: f32,
    #[serde(default = "CircleArgs::default_break_count", alias = "breaks")]
    pub break_count: usize,
    #[serde(default = "CircleArgs::default_break_angle_offset", alias = "angle")]
    pub break_angle_offset: f32,
}
impl CircleArgs {
    fn default() -> Self {
        CircleArgs{
            coil_radius: Self::default_coil_radius(),
            center: Self::default_center(),
            break_count: Self::default_break_count(),
            break_angle_offset: Self::default_break_angle_offset(),
        }
    }
    pub fn default_coil_radius() -> f32 {
        5.0
    }
    pub fn default_center() -> Point {
        Point::new(0.0, 0.0, 0.0)
    }
    pub fn default_break_count() -> usize {
        4
    }
    pub fn default_break_angle_offset() -> f32 {
        0.0
    }
}

impl methods::LayoutMethodTrait for Method {
    /// Get the name of the layout method.
    fn get_method_display_name(&self) -> &'static str {
        "Alternating Circles"
    }

    fn do_layout(&self, surface: &Surface) -> layout::ProcResult<layout::Layout> {

        // Clone the circles
        let mut new_circles = self.circles.clone();

        // Store boundary points
        let boundary_points: Vec<Point> = surface.get_boundary_vertex_indices().iter().map(|v| surface.vertices[*v].point).collect();

        // Store if the coils are on the boundary
        let mut on_boundary = vec![false; new_circles.len()];
        
        // Shrink initial radii to keep the coils within the boundary. Shift center if radius is too small.
        for (coil_id, circle) in new_circles.iter_mut().enumerate() {
            let mut boundary_point = *closest_point(&circle.center, &boundary_points);
            let vec_to_boundary = circle.center - boundary_point;
            let distance_to_boundary = vec_to_boundary.norm();
            if distance_to_boundary < circle.coil_radius {
                let original_center = circle.center;
                circle.center = boundary_point + vec_to_boundary.normalize() * circle.coil_radius;
                circle.center = circle.center - (&circle.center - surface);
                boundary_point = *closest_point(&circle.center, &boundary_points);
                circle.coil_radius = (circle.center - boundary_point).norm();
                if self.warn_on_shift {
                    println!("WARNING: Coil {} too close to boundary, center shifted by |{:.2}| to {:.2} and radius shrunk to {:.2}",
                        coil_id, (original_center - circle.center).norm(), circle.center, circle.coil_radius
                    );
                }
                on_boundary[coil_id] = true;
            }
        }

        // Get initial close coils
        let mut close_coils = 0;
        for (coil_id, coil) in new_circles.iter().enumerate() {
            for (other_coil_id, other_coil) in new_circles.iter().enumerate() {
                if coil_id < other_coil_id {
                    let vec_from_other = coil.center - other_coil.center;
                    let distance_scale = coil.coil_radius + other_coil.coil_radius;
                    let d_rel = vec_from_other.norm() / distance_scale;
                    if d_rel < self.close_cutoff {
                        close_coils += 1;
                    }
                }
            }
        }
            
        // Run a single pass
        let mut layout_out = self.single_pass(surface, &new_circles, false)?;

        // Iterate to automatically decouple
        let mut new_close_coils;
        let mut objective;
        for (i, _) in (0..self.iterations).enumerate() {
            println!("Iteration {}/{}...", (i + 1), self.iterations);

            // Generate step size -- linear decrease currently. TODO Probably should be exponential.
            let step_size = self.initial_step / (1.0 + self.step_decrease * i as f32);

            // Update positions
            new_circles = self.update_positions(
                &new_circles,
                &layout_out,
                surface,
                &boundary_points,
                &mut on_boundary,
                step_size
            );
            layout_out = self.single_pass(surface, &new_circles, false)?;

            // Update radii
            (new_circles, objective, new_close_coils) = self.update_radii(
                &new_circles,
                &layout_out,
                &boundary_points,
                &mut on_boundary,
                step_size
            );
            layout_out = self.single_pass(surface, &new_circles, false)?;

            // Print statistics
            println!("Objective: {:.2}", (objective / new_close_coils as f32).sqrt());
            if close_coils != new_close_coils {
                println!("WARNING: Number of close coils changed! ({} -> {})", close_coils, new_close_coils);
            }
            println!();
            close_coils = new_close_coils;
        }


        // Print statistics
        if self.statistics {
            let mut objective = 0.0;
            let mut close_coils = 0;

            println!("Final Coils:");
            for (coil_id, coil) in layout_out.coils.iter().enumerate() {
                println!("Coil {}: Radius [{:.2}], Center [{:.2}], Length [{:.2}]", coil_id, new_circles[coil_id].coil_radius, coil.center, coil.wire_length());
            }
            println!();

            println!("Coupling factor estimates:");
            for (coil_id, coil) in layout_out.coils.iter().enumerate() {
                for (other_id, other_coil) in layout_out.coils.iter().enumerate() {
                    if coil_id < other_id {
                        let coupling = coil.coupling_factor(other_coil, 1.0);
                        print!("Coil {} to Coil {}:", coil_id, other_id);
                        if coupling.signum() > 0.0 {
                            println!("  {:.3}", coupling);
                        } else {
                            println!(" {:.3}", coupling);
                        }

                        // Track in objective if close
                        let vec_from_other = coil.center - other_coil.center;
                        let distance_scale = new_circles[coil_id].coil_radius + new_circles[other_id].coil_radius;
                        let d_rel = vec_from_other.norm() / distance_scale;
                        if d_rel < self.close_cutoff {
                            close_coils += 1;
                            objective += coupling * coupling * 1.0e6;
                        }
                    }
                }
            }
            println!();

            println!("Self inductance estimates");
            for (coil_id, coil) in layout_out.coils.iter().enumerate() {
                let self_inductance = coil.self_inductance(1.0);
                println!("Coil {}: {:.3}", coil_id, self_inductance);
            }
            println!();

            println!("Objective: {:.2}", (objective / close_coils as f32).sqrt());
            println!();
        }

        if let Some(final_cfg_output) = self.final_cfg_output.as_ref() {
            println!("Writing final cfg...");
            crate::io::save_ser_to(final_cfg_output, &new_circles)?;
        }

        // Add breaks
        println!("Adding breaks...");
        for (coil_id, coil) in layout_out.coils.iter_mut().enumerate() {
            println!("Coil {}/{}...", coil_id + 1, new_circles.len());
            let break_count = new_circles[coil_id].break_count;
            let break_angle_offset_rad = new_circles[coil_id].break_angle_offset * std::f32::consts::PI / 180.0;
            let zero_angle_vector = {
                if coil.normal.normalize().dot(&self.zero_angle_vector.normalize()) < 0.95 {
                    self.zero_angle_vector
                } else {
                    self.backup_zero_angle_vector
                }
            }.normalize();

            add_even_breaks_by_angle(coil, break_count, break_angle_offset_rad, zero_angle_vector)?;
        }
        
        Ok(layout_out)
    }
}

impl Method {

    /// Do a single pass of the alternating circles method
    fn single_pass(&self, surface: &Surface, circles: &Vec::<CircleArgs>, verbose: bool) -> layout::ProcResult<layout::Layout> {
        let mut layout_out = layout::Layout::new();

        // Iterate through the circles
        let wire_radius = self.wire_radius;
        let epsilon = self.epsilon;
        let pre_shift = self.pre_shift;

        for (coil_id, circle_args) in circles.iter().enumerate() {

            if verbose {
                println!("Coil {}/{}...", coil_id + 1, circles.len());
            }
            
            // Grab arguments from the circle arguments
            let coil_radius = circle_args.coil_radius;
            
            // Snap the center to the surface
            let vec_to_surface = &circle_args.center - surface;
            let center = circle_args.center - vec_to_surface;

            // Create the circle through surface intersection with sphere
            let (cid, points, point_normals) = sphere_intersect(surface, center, coil_radius, epsilon);
            let coil_normal = surface.vertices[cid].normal;

            let coil = clean_coil_by_angle(
                center, coil_normal,
                coil_radius, wire_radius,
                points, point_normals,
                pre_shift, false
            )?;

            layout_out.coils.push(coil);
        }

        // Do overlaps
        self.mousehole_overlap(&mut layout_out, circles);

        Ok(layout_out)
    }

    /// Update the positions of the circles
    fn update_positions(&self, 
        circles: &Vec::<CircleArgs>,
        layout_out: &layout::Layout,
        surface: &Surface,
        boundary_points: &Vec::<Point>,
        on_boundary: &mut Vec::<bool>,
        step_size: f32
    ) -> Vec<CircleArgs> {
        let mut new_circles = circles.clone();
        assert!(new_circles.len() == layout_out.coils.len());

        let mut coil_forces = vec![Vec::<GeoVector>::new(); layout_out.coils.len()];

        // Collect radial error 
        let mut radial_err = vec![0.0; layout_out.coils.len()];
        let mut rel_radial_err = vec![0.0; layout_out.coils.len()];
        for (coil_id, circle) in circles.iter().enumerate() {
            radial_err[coil_id] = circle.coil_radius - self.circles[coil_id].coil_radius;
            rel_radial_err[coil_id] = radial_err[coil_id] / self.circles[coil_id].coil_radius;
        }

        // Calculate the forces on each coil
        for (coil_id, coil) in layout_out.coils.iter().enumerate() {

            // Get the parameters that will shift, and their original values
            let mut center = coil.center;
            let original_center = self.circles[coil_id].center;
            let mut radius = circles[coil_id].coil_radius;
            let original_radius = self.circles[coil_id].coil_radius;


            // Check all coils of a higher id than the current coil
            for (other_id, other_coil) in layout_out.coils.iter().enumerate() {
                if other_id > coil_id {

                    // Establish vectors and distances
                    let other_radius = circles[other_id].coil_radius;
                    let vec_from_other = center - other_coil.center;
                    let distance_scale = radius + other_radius;
                    let d_rel = vec_from_other.norm() / distance_scale;

                    // Apply coupling forces from nearby coils
                    if d_rel < self.close_cutoff {
                        let k = coil.coupling_factor(other_coil, 1.0);
                        
                        // Add coupling forces to both coils (split in half)
                        let d_rel_target = d_rel + k;
                        let d_change = d_rel_target * (-radial_err[coil_id] + -radial_err[other_id]) * self.radial_stiffness + k * distance_scale;
                        let offset_force = d_change * vec_from_other.normalize();

                        // Split the change between the two, LESS movement for the one with more radial error.
                        let r_scale = |r_rel_err| -> f32 {f32::powf(2.0, self.radial_stiffness * r_rel_err / self.radius_freedom * d_change.signum())};
                        let total = r_scale(rel_radial_err[coil_id]) + r_scale(rel_radial_err[other_id]);
                        
                        coil_forces[coil_id].push(offset_force * r_scale(rel_radial_err[coil_id]) / total);
                        coil_forces[other_id].push(-offset_force * r_scale(rel_radial_err[other_id]) / total);
                    }
                }
            }
            
            // Find the net force on the center
            let mut delta_c = GeoVector::zero();
            for force in coil_forces[coil_id].iter() {
                let flat_force = force.rej_onto(&coil.normal).normalize() * force.norm();
                delta_c = delta_c + flat_force;
            }

            // Check and update boundary condition
            // If on the boundary, add a normal force keeping the coil from crossing the boundary
            if on_boundary[coil_id] {
                let boundary_point = closest_point(&center, boundary_points);
                let flat_vec_to_boundary = (center - *boundary_point).rej_onto(&coil.normal).normalize();
                let boundary_component = delta_c.proj_onto(&flat_vec_to_boundary);
                if boundary_component.norm() >= 0.0 {
                    delta_c = delta_c - boundary_component;
                } else {
                    on_boundary[coil_id] = false;
                }
            }

            // Update the center
            let center_bound = self.center_freedom * original_radius;
            let total_delta = center + (delta_c.rej_onto(&coil.normal)) - original_center;
            if total_delta.norm() > center_bound {
                delta_c += total_delta.normalize() * (center_bound - total_delta.norm());
            }
            center = center + step_size * delta_c.rej_onto(&coil.normal);

            // If center is too close to the boundary, move it away. Iterate 10 times and then shrink the radius
            let boundary_point = closest_point(&center, boundary_points);
            for i in 0..10 {
                let vec_to_boundary = center - *boundary_point;
                let distance_to_boundary = vec_to_boundary.norm();
                if distance_to_boundary < radius {
                    on_boundary[coil_id] = true;
                    if i < 9 {center = *boundary_point + vec_to_boundary.normalize() * radius;}
                    else {radius = distance_to_boundary;}
                }
            }

            new_circles[coil_id].center = center - (&center - surface);
        }

        // Return the updated circles
        new_circles
    }

    /// Update the radii of the circles
    fn update_radii(
        &self,
        circles: &Vec::<CircleArgs>,
        layout_out: &layout::Layout,
        boundary_points: &Vec::<Point>,
        on_boundary: &mut Vec::<bool>,
        step_size: f32
    ) -> (Vec<CircleArgs>, f32, usize) {
        let mut new_circles = circles.clone();
        assert!(new_circles.len() == layout_out.coils.len());

        // Initialize objective function and number of close coils
        let mut objective = 0.0;
        let mut close_coils = 0;

        // Collect original and min/max radii
        let mut rel_radial_err = vec![0.0; layout_out.coils.len()];
        let mut min_radii = vec![0.0; layout_out.coils.len()];
        let mut max_radii = vec![0.0; layout_out.coils.len()];
        for (coil_id, circle) in circles.iter().enumerate() {
            let original_radius = self.circles[coil_id].coil_radius;
            rel_radial_err[coil_id] = (circle.coil_radius - original_radius) / original_radius;
            min_radii[coil_id] = original_radius * (1.0 - self.radius_freedom);
            max_radii[coil_id] = original_radius * (1.0 + self.radius_freedom);
        }
        
        // Calculate the forces on each coil
        let mut net_radial_change = vec![0.0; layout_out.coils.len()];
        for (coil_id, coil) in layout_out.coils.iter().enumerate() {

            // Get previous values
            let center = coil.center;
            let mut radius = circles[coil_id].coil_radius;

            // Check all coils of a higher id than the current coil
            for (other_id, other_coil) in layout_out.coils.iter().enumerate() {
                if other_id > coil_id {

                    // Establish vectors and distances
                    let other_radius = circles[other_id].coil_radius;
                    let vec_from_other = center - other_coil.center;
                    let distance_scale = radius + other_radius;
                    let d_rel = vec_from_other.norm() / distance_scale;

                    // Apply coupling forces from nearby coils
                    if d_rel < self.close_cutoff {
                        let k = coil.coupling_factor(other_coil, 1.0);

                        // Track close coils and add to objective function
                        close_coils += 1;
                        objective += k * k * 1.0e6;
                        
                        // Add coupling forces to both coils
                        let d_change = k * distance_scale;

                        // Split the change between the two, MORE radial change for the one with more radial error.
                        let r_scale = |r_rel_err| -> f32 {f32::powf(2.0, self.radial_stiffness * r_rel_err / self.radius_freedom * -d_change.signum())};
                        let total = r_scale(rel_radial_err[coil_id]) + r_scale(rel_radial_err[other_id]);

                        net_radial_change[coil_id] -= d_change * r_scale(rel_radial_err[coil_id]) / total;
                        net_radial_change[other_id] -= d_change * r_scale(rel_radial_err[other_id]) / total;
                    }
                }
            }

            // Update the radius
            radius += step_size * net_radial_change[coil_id];
            if radius < min_radii[coil_id] {radius = min_radii[coil_id];}
            else if radius > max_radii[coil_id] {radius = max_radii[coil_id];}

            // Check boundary status, cap at boundary
            let boundary_point = closest_point(&center, boundary_points);
            let distance_to_boundary = (*boundary_point - center).norm();
            if radius > distance_to_boundary {
                radius = distance_to_boundary;
                on_boundary[coil_id] = true;
            } else {
                on_boundary[coil_id] = false;
            }

            new_circles[coil_id].coil_radius = radius;
        }

        (new_circles, objective, close_coils)
    }
        

    /// Do overlaps between the coils
    fn mousehole_overlap(&self, layout_out: &mut layout::Layout, circles: &Vec::<CircleArgs>) {
        let intersections = self.get_intersections(layout_out, 2.0, circles);
        
        // Structure for managing intersecting segments
        #[derive(Clone)]
        struct IntersectionSegment {
            start: usize,
            end: usize,
            length: f32,
            wire_crossings: Vec<f32>,
        }
        
        // Do intersections for each coil
        for (coil_id, coil) in layout_out.coils.iter_mut().enumerate() {

            // Get the length of the coil and the distance around of each point
            let mut point_lengths = vec![0.0; coil.vertices.len()];
            for p in 1..coil.vertices.len() {
                point_lengths[p] = point_lengths[p - 1] + (coil.vertices[p].point - coil.vertices[p - 1].point).norm();
            }
            let coil_length = point_lengths[coil.vertices.len() - 1] + (coil.vertices[0].point - coil.vertices[coil.vertices.len() - 1].point).norm();
    
            // Closure for calculating the distance between two points (wrapping around the coil if necessary)
            let point_distance = |start: usize, end: usize| -> f32 {
                if start < end {
                    point_lengths[end] - point_lengths[start]
                }
                else {
                    point_lengths[end] + (coil_length - point_lengths[start])
                }
            };
    
            // Closure for calculating the length of a segment (adds an extra point to the start and end)
            let padded_segment_length = |start: usize, end: usize| -> f32 {
                let start_anchor = (start + coil.vertices.len() - 1) % coil.vertices.len();
                let end_anchor = (end + 1) % coil.vertices.len();
                point_distance(start_anchor, end_anchor)
            };
            let mut segments = Vec::<IntersectionSegment>::new();
            
            // Get all the intersections between a coil and a coil of higher coil id than it. 
            let mut any_intersections = false;
            for other_id in coil_id+1..circles.len() {
                let other_intersection = &intersections[coil_id][other_id];

                // Ignore loops entirely contained within other loops
                if coil.vertices.len() - other_intersection.len() < 2 {
                    continue;
                }

                if other_intersection.len() > 0 {
                    any_intersections = true;
                    
                    let mut start = other_intersection[0];
                    let mut end;
                    
                    // Check for wraparound
                    let mut i_max = other_intersection.len();
                    if other_intersection[0] == 0 {
                        for (rev_id, p) in other_intersection.iter().rev().enumerate() {
                            if *p != coil.vertices.len() - 1 - rev_id {
                                i_max = other_intersection.len() - rev_id;
                                start = other_intersection[i_max % other_intersection.len()];
                                break;
                            }
                        } 
                    }

                    // Define the segments for this other coil
                    for i in 1..i_max {
                        let p = other_intersection[i];
                        let prev_p = other_intersection[i - 1];
                        if p > prev_p + 1 {
                            end = prev_p;
                            let length = padded_segment_length(start, end);
                            segments.push(IntersectionSegment{
                                start,
                                end,
                                length,
                                wire_crossings: vec![],
                            });
                            start = p;
                        }
                    }
                    end = other_intersection[i_max - 1];
                    let length = padded_segment_length(start, end);
                    segments.push(IntersectionSegment{
                        start,
                        end,
                        length,
                        wire_crossings: vec![],
                    });
                }

                // Update wire crossings
                let other_center = circles[other_id].center;
                let distance_to_other_coil = |p: usize| -> f32 {
                    let point = coil.vertices[p].point;
                    let vec_to_center = point - other_center;
                    vec_to_center.norm()
                };
                let inside_other_coil = |p: usize| -> bool {
                    distance_to_other_coil(p) < circles[other_id].coil_radius
                };
                for segment in segments.iter_mut() {
                    let mut p_prev = segment.start;
                    let mut p = (segment.start + 1) % coil.vertices.len();

                    let in_segment = |x: usize| -> bool {
                        if segment.end < segment.start {
                            x > segment.start || x <= segment.end
                        } else {
                            x > segment.start && x <= segment.end
                        }
                    };

                    while in_segment(p) {
                        if inside_other_coil(p) != inside_other_coil(p_prev) {
                            let length = point_distance(p_prev, p);

                            let d1 = distance_to_other_coil(p_prev).abs();
                            let d2 = distance_to_other_coil(p).abs();

                            let crossing_delta = d1 / (d1 + d2) * length;

                            segment.wire_crossings.push(
                                point_distance(
                                    (segment.start + coil.vertices.len() - 1) % coil.vertices.len(),
                                    p_prev
                                ) + crossing_delta
                            );
                        }
                        p_prev = p;
                        p = (p + 1) % coil.vertices.len();
                    }

                    segment.wire_crossings.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    segment.wire_crossings.dedup();

                    if segment.wire_crossings.len() == 0 {
                        segment.wire_crossings.push(segment.length * 0.5);
                    }
                }
                        
            }
            if !any_intersections {
                continue;
            }

            // Closure for merging the length of two segments
            let merge_length_offset = |start: usize, end: usize| -> f32 {
                let start_anchor = (start + coil.vertices.len() - 1) % coil.vertices.len();
                let end_anchor = (end + coil.vertices.len() - 1) % coil.vertices.len();
                point_distance(start_anchor, end_anchor)
            };
            
            // Closure for merging segments
            let merge_overlap_segments = |first_seg: &IntersectionSegment, second_seg: &IntersectionSegment| -> Option<IntersectionSegment> {
                
                let (first_starts, first_ends) = merge_segments(first_seg.start, first_seg.end, second_seg.start, second_seg.end)?;

                let start_segment = if first_starts { first_seg } else { second_seg };
                let end_segment = if first_ends { first_seg } else { second_seg };

                let start = start_segment.start;
                let end = end_segment.end;

                let length = padded_segment_length(start, end);
                
                let mut wire_crossings = start_segment.wire_crossings.clone();
                let mut end_wire_crossings = end_segment.wire_crossings.clone();
                
                // Offset the end wire crossings by the overlapping length -- merge_length_offset accounts for padding!
                let length_offset = match first_starts == first_ends {
                    false => merge_length_offset(start_segment.start, end_segment.start),
                    true => {
                        let other_segment = if first_starts { second_seg } else { first_seg };
                        merge_length_offset(start_segment.start, other_segment.start)
                    }
                };
                for crossing in end_wire_crossings.iter_mut() {
                    *crossing += length_offset;
                }

                wire_crossings.append(&mut end_wire_crossings);
                wire_crossings.sort_by(|a, b| a.partial_cmp(b).unwrap());
                wire_crossings.dedup();
                Some(IntersectionSegment{
                    start,
                    end,
                    length,
                    wire_crossings,
                })
            };

            // Sort the segments -- first by start, then by length
            segments.sort_by(|a, b| a.start.cmp(&b.start).then(a.length.partial_cmp(&b.length).unwrap()));

            // Merge the segments
            let mut merged_segments = Vec::<IntersectionSegment>::new();
            let mut current_segment = segments[0].clone();
            for seg in segments.into_iter().skip(1) {
                if let Some(merged) = merge_overlap_segments(&current_segment, &seg) {
                    current_segment = merged;
                } else {
                    merged_segments.push(current_segment);
                    current_segment = seg;
                }
            }
            // Handle wrapping
            if merged_segments.len() > 0 {
                if let Some(merged) = merge_overlap_segments(&current_segment, &merged_segments[0]) {
                    merged_segments[0] = merged;
                } else {
                    merged_segments.push(current_segment);
                }
            } else {
                merged_segments.push(current_segment);
            }
                

            // Offset the segments
            for segment in merged_segments.iter_mut() {

                let c = self.clearance + 2.0 * coil.wire_radius;
                // The amount to offset the wire
                let start_tail = segment.wire_crossings[0] / segment.length;
                let end_tail = 1.0 - segment.wire_crossings[segment.wire_crossings.len() - 1] / segment.length;
                let s = c / (2.0 - 2.0_f32.sqrt());
                
                let offset = |l: f32| -> f32 {
                    let l_ratio = l / segment.length;
                    if l_ratio < start_tail {
                        let l_ratio = l_ratio / start_tail;
                        if l_ratio < 0.5 {
                            s * (1.0 - (1.0 - 2.0 * l_ratio * l_ratio).sqrt())
                        } else {
                            s * (1.0 - 2.0_f32.sqrt() + (1.0 - 2.0 * (1.0 - l_ratio) * (1.0 - l_ratio)).sqrt())
                        }
                    } else if l_ratio > (1.0 - end_tail) {
                        let l_ratio = 1.0 - (l_ratio - (1.0 - end_tail)) / (end_tail);
                        if l_ratio < 0.5 {
                            s * (1.0 - (1.0 - 2.0 * l_ratio * l_ratio).sqrt())
                        } else {
                            s * (1.0 - 2.0_f32.sqrt() + (1.0 - 2.0 * (1.0 - l_ratio) * (1.0 - l_ratio)).sqrt())
                        }
                    } else {
                        c
                    }
                };
                // The amount to curve the wire
                let wire_rotation = |l: f32| -> f32 {
                    let l_ratio = l / segment.length;
                    if l_ratio < start_tail {
                        let l_ratio = l_ratio / start_tail;
                        if l_ratio < 0.5 {
                            l_ratio.asin()
                        } else {
                            (1.0 - l_ratio).asin()
                        }
                    } else if l_ratio > (1.0 - end_tail) {
                        let l_ratio = 1.0 - (l_ratio - (1.0 - end_tail)) / (end_tail);
                        if l_ratio < 0.5 {
                            -l_ratio.asin()
                        } else {
                            (l_ratio - 1.0).asin()
                        }
                    } else {
                        0.0
                    }
                };

                let unwrapped_end = if segment.end < segment.start {
                    segment.end + coil.vertices.len()
                }
                else {
                    segment.end
                };

                let start_anchor = (segment.start + coil.vertices.len() - 1) % coil.vertices.len();

                for p in segment.start..=unwrapped_end {
                    let pid = p % coil.vertices.len();
                    coil.vertices[pid].point = coil.vertices[pid].point - coil.vertices[pid].surface_normal * offset(point_distance(start_anchor, pid));
                    let surface_tangent = (coil.vertices[pid].point - coil.center).rej_onto(&coil.vertices[pid].surface_normal).normalize();
                    coil.vertices[pid].wire_radius_normal = 
                        coil.vertices[pid].wire_radius_normal
                        .rotate_around(&surface_tangent, wire_rotation(point_distance(start_anchor, pid)));
                }
            }  
        }
    }

    /// Get the adjacency matrix for the circles laid out on the surface
    #[allow(dead_code)]
    fn get_adjacency(&self, surface: &Surface, circles: &Vec::<CircleArgs>) -> Vec<Vec<bool>> {
        let mut adjacency: Vec<Vec<bool>> = vec![vec![false; circles.len()]; circles.len()];
        for vertex in surface.vertices.iter() {
            let point = vertex.point;
            for (i, circle) in circles.iter().enumerate() {
                let center = circle.center;
                let radius = circle.coil_radius;
                if (point - center).norm() < radius {
                    for (j, other_circle) in circles.iter().enumerate() {
                        if i != j {
                            let other_center = other_circle.center;
                            let other_radius = other_circle.coil_radius;
                            if (point - other_center).norm() < other_radius {
                                adjacency[i][j] = true;
                                adjacency[j][i] = true;
                            }
                        }
                    }
                }
            }
        }
        adjacency
    }

    /// Get a matrix of vectors of intersection points between cleaned coils
    #[allow(dead_code)]
    fn get_intersections(&self, intersecting_layout: &layout::Layout, clearance_scale: f32, circles: &Vec::<CircleArgs>) -> Vec<Vec<Vec<usize>>> {
        let mut intersections: Vec<Vec<Vec<usize>>> = vec![vec![vec![]; circles.len()]; circles.len()];
        for (i, coil) in intersecting_layout.coils.iter().enumerate() {
            for (j, other_coil) in intersecting_layout.coils.iter().enumerate() {
                if i != j {
                    for (k, vertex) in coil.vertices.iter().enumerate() {
                        if ((vertex.point - other_coil.center).norm() - circles[j].coil_radius).abs() < 
                            (coil.wire_radius + other_coil.wire_radius + self.clearance) * clearance_scale {
                            
                            intersections[i][j].push(k);
                        }
                    }
                }
            }
        }
        intersections
    }
}

mod debug {
    use super::*;

    #[allow(dead_code)]
    pub fn dump_yaml(method: &Method) {
        let s = serde_yaml::to_string(&method).unwrap();
        println!("{}", s);
    }
}
