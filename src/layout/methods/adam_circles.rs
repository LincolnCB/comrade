/*!
*   Circular layout method with symmetry, using numerically calculated gradient of coil coupling.
*   Uses the ADAM optimization algorithm.
*   Inclusion of symmetry plane assumes that the surface is roughly symmetric about the plane.
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
use itertools::concat;

/// ADAM Circles method struct.
/// This struct contains all the parameters for the ADAM Circles layout method.
#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Method {
    // Optional symmetry plane
    #[serde(default = "Method::default_symmetry_plane", alias = "plane")]
    pub symmetry_plane: Option<Plane>,
    #[serde(default = "Method::default_layout_in_path", rename = "layout_in", alias = "static_layout")]
    pub layout_in_path: Option<String>,

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
    pub fn example_symmetry_plane() -> Option<Plane> {
        Some(Plane::from_normal_and_offset(GeoVector::xhat(), 0.0))
    }
    pub fn default_symmetry_plane() -> Option<Plane> {
        None
    }
    pub fn example_layout_in_path() -> Option<String> {
        Some("PATH/TO/INITIAL/CFG.json".to_string())
    }
    pub fn default_layout_in_path() -> Option<String> {
        None
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
        0
    }
    pub fn example_iterations() -> usize {
        5
    }
    pub fn default_step_size() -> f32 {
        64.0
    }
    pub fn default_first_moment_decay() -> f32 {
        0.9
    }
    pub fn default_second_moment_decay() -> f32 {
        0.999
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
    pub fn default_radius_reg() -> f32 {
        1.0
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
            symmetry_plane: Self::example_symmetry_plane(),
            layout_in_path: Self::example_layout_in_path(),

            circles: vec![CircleArgs::default(); 2],
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

/// Single element arguments
#[derive(Debug, Clone, Copy)]
#[derive(Serialize, Deserialize)]
pub struct CircleArgs {
    pub center: Point,
    #[serde(default = "CircleArgs::default_coil_radius", alias = "radius")]
    pub coil_radius: f32,
    #[serde(default = "CircleArgs::default_break_count", alias = "breaks")]
    pub break_count: usize,
    #[serde(default = "CircleArgs::default_break_angle_offset", alias = "angle")]
    pub break_angle_offset: f32,
    #[serde(default = "CircleArgs::default_on_symmetry_plane", alias = "on_sym")]
    pub on_symmetry_plane: bool,
}
impl CircleArgs {
    fn default() -> Self {
        CircleArgs{
            coil_radius: Self::default_coil_radius(),
            center: Self::default_center(),
            break_count: Self::default_break_count(),
            break_angle_offset: Self::default_break_angle_offset(),
            on_symmetry_plane: Self::default_on_symmetry_plane(),
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
    pub fn default_on_symmetry_plane() -> bool {
        false
    }
}

/// ADAM Circles moment struct.
#[derive(Debug, Clone)]
struct Moment {
    first_center: GeoVector,
    second_center: GeoVector,
    first_radius: f32,
    second_radius: f32,
    first_bias_correction: f32,
    second_bias_correction: f32,
}
impl Moment {
    fn new() -> Self {
        Moment{
            first_center: GeoVector::zero(),
            second_center: GeoVector::zero(),
            first_radius: 0.0,
            second_radius: 0.0,
            first_bias_correction: 1.0,
            second_bias_correction: 1.0,
        }
    }

    fn update(&mut self, center_grad: GeoVector, radius_grad: f32, first_moment_decay: f32, second_moment_decay: f32) {
        self.first_center = first_moment_decay * self.first_center + (1.0 - first_moment_decay) * center_grad;
        self.second_center = second_moment_decay * self.second_center + (1.0 - second_moment_decay) * center_grad.el_powf(2.0);
        self.first_radius = first_moment_decay * self.first_radius + (1.0 - first_moment_decay) * radius_grad;
        self.second_radius = second_moment_decay * self.second_radius + (1.0 - second_moment_decay) * radius_grad * radius_grad;
        self.first_bias_correction *= first_moment_decay;
        self.second_bias_correction *= second_moment_decay;
    }

    fn get_center_step(&self) -> GeoVector {
        let first_unbiased = self.first_center / (1.0 - self.first_bias_correction);
        let second_unbiased = self.second_center / (1.0 - self.second_bias_correction);
        first_unbiased.el_div(&second_unbiased.el_powf(0.5).el_add(1.0e-8))
    }

    fn get_radius_step(&self) -> f32 {
        let first_unbiased = self.first_radius / (1.0 - self.first_bias_correction);
        let second_unbiased = self.second_radius / (1.0 - self.second_bias_correction);
        first_unbiased / (second_unbiased.powf(0.5) + 1.0e-8)
    }
}

impl methods::LayoutMethodTrait for Method {
    /// Get the name of the layout method.
    fn get_method_display_name(&self) -> &'static str {
        "ADAM Circles (Optional Symmetry)"
    }

    fn do_layout(&self, surface: &Surface) -> layout::ProcResult<layout::Layout> {

        // Initialize potential symmetrical circles
        let mut sym_circles = Vec::<CircleArgs>::new();
        let mut pos_circles = Vec::<CircleArgs>::new();
        let mut neg_circles = Vec::<CircleArgs>::new();

        // Load the static layout if provided
        let static_layout = if let Some(layout_in_path) = &self.layout_in_path {
            println!("Loading initial layout...");
            Some(crate::io::load_deser_from::<layout::Layout>(layout_in_path)?)
        } else {
            None
        };

        // Collect and clone the circles, with extra effort for symmetry
        let original_circles = if let Some(symmetry_plane) = &self.symmetry_plane {
            // Separate the coils by their symmetry
            for (circle_num, circle) in self.circles.iter().enumerate() {
                if circle.on_symmetry_plane {
                    // Make sure the circle is on the symmetry plane
                    let mut circle = circle.clone();
                    if symmetry_plane.distance_to_point(&circle.center).abs() > self.epsilon {
                        println!("WARNING: Circle {} more than epsilon ({}) from symmetry plane, moving to symmetry plane", circle_num, self.epsilon);
                    }
                    circle.center = symmetry_plane.project_point(&circle.center);
                    sym_circles.push(circle);
                } else {
                    // Make sure the circle is on the right side of the symmetry plane
                    let mut circle = circle.clone();
                    if symmetry_plane.distance_to_point(&circle.center) < 0.0 {
                        println!("WARNING: Circle {} on wrong side of symmetry plane, flipping", circle_num);
                        circle.center = circle.center.reflect_across(&symmetry_plane);
                    }
                    if symmetry_plane.distance_to_point(&circle.center).abs() < self.epsilon {
                        println!("WARNING: Circle {} close to symmetry plane, may cause issues", circle_num);
                    }
                    pos_circles.push(circle);

                    // Add the flipped circle
                    let mut neg_circle = circle.clone();
                    neg_circle.center = neg_circle.center.reflect_across(&symmetry_plane);
                    neg_circles.push(neg_circle);
                }
            }

            // Collect all the circles
            concat(vec![sym_circles.clone(), pos_circles.clone(), neg_circles.clone()])
        } else {
            // Copy the circles
            self.circles.clone()
        };

        let mut new_circles = original_circles.clone();

        // Store boundary points
        let boundary_points: Vec<Point> = surface.get_boundary_vertex_indices().iter()
            .map(|v| surface.vertices[*v].point).collect();

        // Store if the coils are on the boundary
        let mut on_boundary = vec![false; new_circles.len()];
        
        // Shrink initial radii to keep the coils within the boundary. Shift center if radius is too small.
        let pos_offset = sym_circles.len();
        let neg_offset = pos_offset + pos_circles.len();
        let total_coil_count = new_circles.len();
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
                if let Some(symmetry_plane) = &self.symmetry_plane {
                    match coil_id {
                        n if (0..pos_offset).contains(&n) => {
                            sym_circles[coil_id].center = circle.center;
                            if self.warn_on_shift {
                                println!("WARNING: Coil {} too close to boundary, center shifted by |{:.2}| to {:.2} and radius shrunk to {:.2}",
                                    coil_id, (original_center - circle.center).norm(), circle.center, circle.coil_radius
                                );
                            }
                        },
                        n if (pos_offset..neg_offset).contains(&n) => {
                            pos_circles[coil_id - pos_offset].center = circle.center;
                            neg_circles[coil_id - pos_offset].center = circle.center.reflect_across(&symmetry_plane);
                            if self.warn_on_shift {
                                println!("WARNING: Coil {} too close to boundary, center shifted by |{:.2}| to {:.2} and radius shrunk to {:.2}",
                                    coil_id, (original_center - circle.center).norm(), circle.center, circle.coil_radius
                                );
                            }
                        },
                        n if (neg_offset..total_coil_count).contains(&n) => {
                            neg_circles[coil_id - neg_offset].center = circle.center;
                            pos_circles[coil_id - neg_offset].center = circle.center.reflect_across(&symmetry_plane);
                            if self.warn_on_shift {
                                println!("WARNING: Coil {} (reflection of coil {}) too close to boundary, center shifted by |{:.2}| to {:.2} and radius shrunk to {:.2}",
                                    coil_id, coil_id - neg_offset, (original_center - circle.center).norm(), circle.center, circle.coil_radius
                                );
                            }
                        },
                        _ => unreachable!(),
                    }
                } else {
                    if self.warn_on_shift {
                        println!("WARNING: Coil {} too close to boundary, center shifted by |{:.2}| to {:.2} and radius shrunk to {:.2}",
                            coil_id, (original_center - circle.center).norm(), circle.center, circle.coil_radius
                        );
                    }
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

            // Include static coils
            if let Some(static_layout) = static_layout.as_ref() {
                for static_coil in static_layout.coils.iter() {
                    for vertex in static_coil.vertices.iter() {
                        let vec_from_static = coil.center - vertex.point;
                        if vec_from_static.norm() / coil.coil_radius < self.close_cutoff {
                            close_coils += 1;
                            break;
                        }
                    }
                }
            }
        }
            
        // Run a single pass
        let mut layout_out = if let Some(symmetry_plane) = &self.symmetry_plane {
            self.lay_out_coils_sym(
                surface,
                symmetry_plane,
                &sym_circles,
                &pos_circles,
                &neg_circles,
                false)?
        } else {
            self.lay_out_coils(surface, &new_circles, false)?
        };

        // Store the previous and best layouts
        let mut prev_layout_out = layout_out.clone();
        let mut best_layout_out = layout_out.clone();

        // Iterate to automatically decouple
        let mut new_close_coils;
        let mut objective;
        let mut best_rms = std::f32::INFINITY;
        let mut moments = vec![Moment::new(); new_circles.len()];
        for i in 0..self.iterations {
            println!();
            println!("Iteration {}/{}...", (i + 1), self.iterations);

            if let Some(symmetry_plane) = &self.symmetry_plane {
                // Update circles symmetrically
                // Objective is calculated from the PREVIOUS layout, not the updated one 
                // (the one that goes in to the function, not the one that comes out of it)
                (sym_circles, pos_circles, neg_circles, objective, new_close_coils)  = self.update_circles_sym(
                    &sym_circles,
                    &pos_circles,
                    &neg_circles,
                    &original_circles,
                    &layout_out,
                    &static_layout,
                    surface,
                    symmetry_plane,
                    &boundary_points,
                    &mut on_boundary,
                    &mut moments
                );
                layout_out = self.lay_out_coils_sym(
                    surface,
                    symmetry_plane,
                    &sym_circles,
                    &pos_circles,
                    &neg_circles,
                    false
                )?;
                new_circles = concat(vec![sym_circles.clone(), pos_circles.clone(), neg_circles.clone()]);
            } else {
                // Update circles
                (new_circles, objective, new_close_coils) = self.update_circles(
                    &new_circles,
                    &original_circles,
                    &layout_out,
                    &static_layout,
                    surface,
                    &boundary_points,
                    &mut on_boundary,
                    &mut moments
                );
                layout_out = self.lay_out_coils(surface, &new_circles, false)?;
            }

            // Store the best layout
            if (objective / new_close_coils as f32).sqrt() < best_rms {
                best_layout_out = prev_layout_out.clone();
                best_rms = (objective / new_close_coils as f32).sqrt();
            }
            prev_layout_out = layout_out.clone();

            // Print statistics
            println!("Starting RMS Coupling: {:.2}", (objective / new_close_coils as f32).sqrt());
            if close_coils != new_close_coils {
                println!("WARNING: Number of close coils changed! ({} -> {})", close_coils, new_close_coils);
            }
            close_coils = new_close_coils;
        }

        // One final objective calculation for the best layout
        let self_inductances;
        let mutual_inductances;
        let static_mutual_inductances;
        (objective, close_coils, self_inductances, mutual_inductances, static_mutual_inductances) = 
            self.get_statistics(
                &new_circles,
                &layout_out,
                &static_layout,
                self.statistics_level > 1,
                self.statistics_level > 2
            );
        println!("Final RMS Coupling: {:.2}", (objective / close_coils as f32).sqrt());
        if (objective / close_coils as f32).sqrt() < best_rms {
            best_layout_out = layout_out.clone();
            best_rms = (objective / close_coils as f32).sqrt();
        }
        println!("Best RMS Coupling: {:.2}", best_rms.sqrt());
        println!();

        // Print statistics
        if self.statistics_level > 0 {
            println!("Final Coils:");
            for (coil_id, coil) in best_layout_out.coils.iter().enumerate() {
                println!("Coil {}: Radius [{:.2}], Center [{:.2}], Length [{:.2}]", coil_id, new_circles[coil_id].coil_radius, coil.center, coil.wire_length());
            }
            println!();
            
            println!("Self-inductances:");
            for (coil_id, self_inductance) in self_inductances.iter().enumerate() {
                println!("Coil {}: {:.2}nH", coil_id, self_inductance);
            }
            println!();

            println!("Mutual inductance estimates:");
            for (coil_id, mutual_ind_vec) in mutual_inductances.iter().enumerate() {
                for (other_id, m) in mutual_ind_vec.iter() {
                    println!("Coil {} with Coil {}: {:.2}nH", coil_id, other_id, m);
                }
            }
            println!();

            if let Some(_) = static_layout.as_ref() {
                println!("Mutual inductance estimates with static coils:");
                for (coil_id, static_mutual_ind_vec) in static_mutual_inductances.iter().enumerate() {
                    for (static_id, m) in static_mutual_ind_vec.iter() {
                        println!("Coil {} with Static Coil {}: {:.2}nH", coil_id, static_id, m);
                    }
                }
                println!();
            }
        }

        if let Some(final_cfg_output) = self.final_cfg_output.as_ref() {
            println!("Writing final cfg...");
            crate::io::save_ser_to(final_cfg_output, &new_circles)?;
        }

        // Add breaks
        println!("Adding breaks...");
        for (coil_id, coil) in best_layout_out.coils.iter_mut().enumerate() {
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
        
        Ok(best_layout_out)
    }
}

impl Method {

    /// Do a single pass of spherical intersection on the circles
    fn lay_out_coils(
        &self,
        surface: &Surface,
        circles: &Vec::<CircleArgs>,
        verbose: bool
    ) -> layout::ProcResult<layout::Layout> {

        let mut layout_out = layout::Layout::new();

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
            let (cid, points, point_normals) = sphere_intersect(surface, center, coil_radius, self.epsilon);
            let coil_normal = surface.vertices[cid].normal;

            let coil = clean_coil_by_angle(
                center,
                coil_normal,
                coil_radius, 
                self.wire_radius,
                points,
                point_normals,
                self.pre_shift,
                false
            )?;

            layout_out.coils.push(coil);
        }

        // Do overlaps
        self.mousehole_overlap(&mut layout_out, circles);

        Ok(layout_out)
    }

    /// Do a single pass of symmetric coil intersection
    fn lay_out_coils_sym(
        &self, 
        surface: &Surface, 
        symmetry_plane: &Plane,
        sym_circles: &Vec::<CircleArgs>, 
        pos_circles: &Vec::<CircleArgs>, 
        neg_circles: &Vec::<CircleArgs>, 
        verbose: bool
    ) -> layout::ProcResult<layout::Layout> {

        let mut layout_out = layout::Layout::new();

        // Create the coils for the on-symmetry circles
        for (_, circle_args) in sym_circles.iter().enumerate() {
            
            // Grab arguments from the circle arguments
            let coil_radius = circle_args.coil_radius;
            let center = circle_args.center;

            // Create the circle through surface intersection with sphere
            let (cid, points, point_normals) =
                sphere_intersect(surface, center, coil_radius, self.epsilon);
            let coil_normal = surface.vertices[cid].normal.normalize();

            if verbose { println!("Uncleaned point count: {}", points.len()) };

            let coil = clean_coil_by_angle(
                center,
                coil_normal,
                coil_radius,
                self.wire_radius,
                points,
                point_normals,
                self.pre_shift,
                false
            )?;
    
            if verbose { println!("Cleaned point count: {}", coil.vertices.len()) };
    
            layout_out.coils.push(coil);
        }

        // Create the coils for the positive circles
        for (_, circle_args) in pos_circles.iter().enumerate() {
            
            // Grab arguments from the circle arguments
            let coil_radius = circle_args.coil_radius;
            let center = circle_args.center;

            // Create the circle through surface intersection with sphere
            let (cid, points, point_normals) =
                sphere_intersect(surface, center, coil_radius, self.epsilon);
            let coil_normal = surface.vertices[cid].normal.normalize();

            if verbose { println!("Uncleaned point count: {}", points.len()) };

            let coil = clean_coil_by_angle(
                center,
                coil_normal,
                coil_radius,
                self.wire_radius,
                points,
                point_normals,
                self.pre_shift,
                false
            )?;
    
            if verbose { println!("Cleaned point count: {}", coil.vertices.len()) };
    
            layout_out.coils.push(coil);
        }

        // Create the coils for the flipped circles
        for i in 0..pos_circles.len() {
            let coil = &layout_out.coils[sym_circles.len() + i];
            let neg_coil = layout::Coil::new(
                coil.center.reflect_across(&symmetry_plane),
                coil.normal.reflect_across(&symmetry_plane.get_normal()),
                coil.vertices.iter().map(|vertex| vertex.point.reflect_across(&symmetry_plane)).rev().collect(),
                coil.wire_radius,
                coil.vertices.iter().map(|vertex| vertex.surface_normal.reflect_across(&symmetry_plane.get_normal())).rev().collect()
            )?;
            layout_out.coils.push(neg_coil);
        }

        // Do overlaps
        let circles = concat(vec![sym_circles.clone(), pos_circles.clone(), neg_circles.clone()]);
        self.mousehole_overlap(&mut layout_out, &circles);

        Ok(layout_out)
    }

    /// Update the circle parameters
    fn update_circles(
        &self, 
        circles: &Vec::<CircleArgs>,
        original_circles: &Vec::<CircleArgs>,
        layout_out: &layout::Layout,
        static_layout: &Option<layout::Layout>,
        surface: &Surface,
        boundary_points: &Vec::<Point>,
        on_boundary: &mut Vec::<bool>,
        moments: &mut Vec::<Moment>
    ) -> (Vec<CircleArgs>, f32, usize) {

        let mut new_circles = circles.clone();
        assert!(new_circles.len() == layout_out.coils.len());

        // Initialize objective function and number of close coils
        let mut objective = 0.0;
        let mut close_coils = 0;

        let mut self_inductances = vec![0.0; layout_out.coils.len()];
        let mut static_self_inductances: Vec::<Option<f32>> = if let Some(static_layout) = static_layout.as_ref() {
            vec![None; static_layout.coils.len()]
        } else {
            vec![]
        };

        // Collect radial error and self inductance
        let mut radial_err = vec![0.0; layout_out.coils.len()];
        let mut rel_radial_err = vec![0.0; layout_out.coils.len()];
        let mut min_radii = vec![0.0; layout_out.coils.len()];
        let mut max_radii = vec![0.0; layout_out.coils.len()];
        for (coil_id, circle) in circles.iter().enumerate() {
            let original_radius = original_circles[coil_id].coil_radius;
            radial_err[coil_id] = circle.coil_radius - original_radius;
            rel_radial_err[coil_id] = (circle.coil_radius - original_radius) / original_radius;
            min_radii[coil_id] = original_radius * (1.0 - self.radius_freedom);
            max_radii[coil_id] = original_radius * (1.0 + self.radius_freedom);
            self_inductances[coil_id] = layout_out.coils[coil_id].self_inductance(1.0);
        }

        // Calculate the updates for each coil
        let mut center_grads = vec![Vec::<GeoVector>::new(); layout_out.coils.len()];
        let mut radial_grads = vec![0.0; layout_out.coils.len()];
        for (coil_id, coil) in layout_out.coils.iter().enumerate() {

            // Initialize with the regularization term for radius
            radial_grads[coil_id] += -2.0 * self.radius_reg * rel_radial_err[coil_id];

            // Get the parameters that will shift, and their original values
            let mut center = coil.center;
            let original_center = original_circles[coil_id].center;
            let mut radius = circles[coil_id].coil_radius;
            let original_radius = original_circles[coil_id].coil_radius;

            // Check all coils of a higher id than the current coil
            for (other_id, other_coil) in layout_out.coils.iter().enumerate() {
                if other_id != coil_id {

                    // Establish vectors and distances
                    let other_radius = circles[other_id].coil_radius;
                    let vec_from_other = center - other_coil.center;

                    // Apply coupling forces from nearby coils
                    if vec_from_other.norm() / (radius + other_radius) < self.close_cutoff {

                        // Track close coils and add to objective function
                        if other_id > coil_id {

                            // Get coupling and gradient wrt center and radius
                            let (m, dx, dy, dz, dr) = coil.mutual_inductance_full(other_coil, 1.0);

                            // Track the objective function and close coils
                            close_coils += 1;
                            objective += m * m * 1.0e6 / (self_inductances[coil_id] * self_inductances[other_id]);

                            // Track the gradients relative to the center location
                            // dk^2/dx = 2k * dk/dx = 2(m/sqrt(L1L2)) * dm/dx / sqrt(L1L2) = 2m * dm/dx / L1L2
                            let adjustment = 2.0 * m * GeoVector::new(dx, dy, dz)
                            / (self_inductances[coil_id] * self_inductances[other_id]);

                            // Add the force to the coil
                            center_grads[coil_id].push(adjustment);
                            center_grads[other_id].push(-adjustment);

                            // Track the gradient relative to the radius
                            radial_grads[coil_id] += 2.0 * m * dr / (self_inductances[coil_id] * self_inductances[other_id]);
                        } else {

                            // Just get the gradient wrt radius
                            let (m, dr) = coil.mutual_inductance_dradius(other_coil, 1.0);

                            // Track the gradient relative to the radius
                            radial_grads[coil_id] += 2.0 * m * dr / (self_inductances[coil_id] * self_inductances[other_id]);
                        }
                    }
                }
            }

            // Check all static coils
            if let Some(static_layout) = static_layout.as_ref() {
                for (static_id, static_coil) in static_layout.coils.iter().enumerate() {
                    let mut close = false;

                    // Calculate proximity exactly to allow for non-spherical static coils
                    for vertex in static_coil.vertices.iter() {
                        let vec_from_static = center - vertex.point;
                        if vec_from_static.norm() / radius < self.close_cutoff {
                            close = true;
                            break;
                        }
                    }

                    // Apply coupling forces from nearby static coil
                    if close {

                        // Get coupling and gradient
                        let (m, dx, dy, dz, dr) = coil.mutual_inductance_full(static_coil, 1.0);   

                        // Grab the self inductance, if not already calculated
                        if static_self_inductances[static_id].is_none() {
                            static_self_inductances[static_id] = Some(coil.self_inductance(1.0));
                        }

                        // Track the objective function and close coils
                        close_coils += 1;
                        objective += m * m * 1.0e6 / (self_inductances[coil_id] * static_self_inductances[static_id].unwrap());

                        // Track the gradients relative to the center location
                        // dk^2/dx = 2k * dk/dx = 2(m/sqrt(L1L2)) * dm/dx / sqrt(L1L2) = 2m * dm/dx / L1L2
                        center_grads[coil_id].push(
                            2.0 * m * GeoVector::new(dx, dy, dz)
                            / (self_inductances[coil_id] * static_self_inductances[static_id].unwrap())
                        );

                        // Track the gradient relative to the radius
                        radial_grads[coil_id] += 2.0 * m * dr / (self_inductances[coil_id] * static_self_inductances[static_id].unwrap());
                    }
                }
            }
            
            // Find the proximal operator on the center
            let mut prox_center = GeoVector::zero();
            for grad in center_grads[coil_id].iter() {
                prox_center = prox_center + grad.rej_onto(&coil.normal);
            }

            // Check and update center boundary condition
            // If on the boundary, add a normal force keeping the coil from crossing the boundary
            if on_boundary[coil_id] {
                let boundary_point = closest_point(&center, boundary_points);
                let flat_vec_to_boundary = (center - *boundary_point).rej_onto(&coil.normal).normalize();
                let boundary_component = prox_center.proj_onto(&flat_vec_to_boundary);
                if boundary_component.norm() >= 0.0 {
                    prox_center = prox_center - boundary_component;
                } else {
                    on_boundary[coil_id] = false;
                }
            }

            // Update moments
            moments[coil_id].update(prox_center, radial_grads[coil_id], self.first_moment_decay, self.second_moment_decay);
            
            // Update the center
            let mut center_step = -self.step_size * moments[coil_id].get_center_step();
            let center_bound = self.center_freedom * original_radius;
            let total_delta = center + (center_step.rej_onto(&coil.normal)) - original_center;
            if total_delta.norm() > center_bound {
                center_step += total_delta.normalize() * (center_bound - total_delta.norm());
            }
            center = center + center_step.rej_onto(&coil.normal);

            // Update the radius
            radius += -self.step_size * moments[coil_id].get_radius_step();
            if radius < min_radii[coil_id] {radius = min_radii[coil_id];}
            else if radius > max_radii[coil_id] {radius = max_radii[coil_id];}

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

            // Project the center onto the surface
            new_circles[coil_id].center = center - (&center - surface);
            new_circles[coil_id].coil_radius = radius;
        }

        // Return the updated circles
        (new_circles, objective, close_coils)
    }
    
    /// Update the parameters of the circles with symmetry
    /// Returns the symmetric, positive, and negative circles
    fn update_circles_sym(
        &self,
        sym_circles: &Vec::<CircleArgs>,
        pos_circles: &Vec::<CircleArgs>,
        neg_circles: &Vec::<CircleArgs>,
        original_circles: &Vec::<CircleArgs>,
        layout_out: &layout::Layout,
        static_layout: &Option<layout::Layout>,
        surface: &Surface,
        symmetry_plane: &Plane,
        boundary_points: &Vec::<Point>,
        on_boundary: &mut Vec::<bool>,
        moments: &mut Vec::<Moment>
    ) -> (Vec<CircleArgs>, Vec<CircleArgs>, Vec<CircleArgs>, f32, usize) {

        let old_circles = concat(vec![sym_circles.clone(), pos_circles.clone(), neg_circles.clone()]);

        // Update the circles
        let (new_circles, objective, close_coils) = self.update_circles(
            &old_circles,
            original_circles,
            layout_out,
            static_layout,
            surface,
            boundary_points,
            on_boundary,
            moments
        );

        // Split the circles back into their respective groups
        let mut new_sym_circles = Vec::<CircleArgs>::new();
        let mut new_pos_circles = Vec::<CircleArgs>::new();
        let mut new_neg_circles = Vec::<CircleArgs>::new();
        for (i, circle) in new_circles.iter().enumerate() {
            if i < sym_circles.len() {
                new_sym_circles.push(*circle);
            } else if i < sym_circles.len() + pos_circles.len() {
                new_pos_circles.push(*circle);
            } else {
                new_neg_circles.push(*circle);
            }
        }

        // Project the symmetric circles onto the symmetry plane, then again onto the surface
        for circle in new_sym_circles.iter_mut() {
            circle.center = symmetry_plane.project_point(&circle.center);
            circle.center = circle.center - (&circle.center - surface).rej_onto(&symmetry_plane.get_normal());
        }

        // Average the positive and negative circles (flipped) to keep them symmetric
        for (pos_circle, neg_circle) in new_pos_circles.iter_mut().zip(new_neg_circles.iter_mut()) {
            pos_circle.center = ((GeoVector::from(pos_circle.center) + GeoVector::from(neg_circle.center.reflect_across(&symmetry_plane))) / 2.0).into();
            neg_circle.center = pos_circle.center.reflect_across(&symmetry_plane);
        }

        // Return the updated circles
        (new_sym_circles, new_pos_circles, new_neg_circles, objective, close_coils)
    }

    /// Get the final statistics
    /// Returns: Objective function, number of close coils, self-inductances, mutual inductances, and static mutual inductances
    fn get_statistics(
        &self,
        circles: &Vec::<CircleArgs>,
        layout_out: &layout::Layout,
        static_layout: &Option<layout::Layout>,
        calc_all_nonstatic: bool,
        calc_all_static: bool
    ) -> (f32, usize, Vec<f32>, Vec<Vec<(usize, f32)>>, Vec<Vec<(usize, f32)>>) {

        // Initialize objective function and number of close coils
        let mut objective = 0.0;
        let mut close_coils = 0;

        let mut self_inductances = vec![0.0; layout_out.coils.len()];
        let mut static_self_inductances: Vec::<Option<f32>> = if let Some(static_layout) = static_layout.as_ref() {
            vec![None; static_layout.coils.len()]
        } else {
            vec![]
        };

        let mut mutual_inductances: Vec<Vec<(usize, f32)>> = vec![vec![]; layout_out.coils.len()];
        let mut static_mutual_inductances: Vec<Vec<(usize, f32)>> = vec![vec![]; layout_out.coils.len()];

        // Calculate all the self inductances
        for (coil_id, coil) in layout_out.coils.iter().enumerate() {
            self_inductances[coil_id] = coil.self_inductance(1.0);
        }

        // Calculate the objective contribution for each coil
        for (coil_id, coil) in layout_out.coils.iter().enumerate() {

            // Check all coils of a higher id than the current coil
            for (other_id, other_coil) in layout_out.coils.iter().enumerate() {
                if other_id > coil_id {

                    // Establish vectors and distances
                    let close = (coil.center - other_coil.center).norm() 
                        / (circles[coil_id].coil_radius + circles[other_id].coil_radius) < self.close_cutoff;

                    // Count objective from close coils
                    if calc_all_nonstatic || close {

                        // Get coupling
                        let m = coil.mutual_inductance(other_coil, 1.0);

                        // Track the objective function and close coils
                        if close {
                            close_coils += 1;
                            objective += m * m * 1.0e6 / (self_inductances[coil_id] * self_inductances[other_id]);
                        }

                        // Track the mutual inductance
                        mutual_inductances[coil_id].push((other_id, m));
                    }
                }
            }

            // Check all static coils
            if let Some(static_layout) = static_layout.as_ref() {
                for (static_id, static_coil) in static_layout.coils.iter().enumerate() {
                    let mut close = false;

                    // Calculate proximity exactly to allow for non-spherical static coils
                    for vertex in static_coil.vertices.iter() {
                        let vec_from_static = coil.center - vertex.point;
                        if vec_from_static.norm() / circles[coil_id].coil_radius < self.close_cutoff {
                            close = true;
                            break;
                        }
                    }

                    // Track objective from close static coils
                    if calc_all_static || close {

                        // Get coupling
                        let m = coil.mutual_inductance(static_coil, 1.0);   

                        // Grab the self inductance, if not already calculated
                        if static_self_inductances[static_id].is_none() {
                            static_self_inductances[static_id] = Some(coil.self_inductance(1.0));
                        }

                        // Track the objective function and close coils
                        if close{
                            close_coils += 1;
                            objective += m * m * 1.0e6 / (self_inductances[coil_id] * static_self_inductances[static_id].unwrap());
                        }
                    
                        // Track the mutual inductance
                        static_mutual_inductances[coil_id].push((static_id, m));
                    }
                }
            }
        }

        // Return the objective function and number of close coils
        (objective, close_coils, self_inductances, mutual_inductances, static_mutual_inductances)
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
