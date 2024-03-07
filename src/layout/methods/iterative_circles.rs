/*!
*   Iterative Circles Method
*
*
!*/

use crate::{
    layout,
    args
};
use layout::methods;
use layout::geo_3d::*;
use methods::helper::{sphere_intersect, clean_coil_by_angle, merge_segments};

use serde::{Serialize, Deserialize};

/// Iterative Circles Method struct.
/// This struct contains all the parameters for the Iterative Circles layout method.
#[derive(Debug)]
pub struct Method {
    /// Arguments for the Iterative Circles method.
    method_args: MethodCfg,
}
impl Method {
    pub fn new() -> args::ProcResult<Self> {
        Ok(Method{method_args: MethodCfg::default()})
    }
}

/// Deserializer from yaml method cfg file
#[derive(Debug, Serialize, Deserialize)]
struct MethodCfg {
    circles: Vec<CircleArgs>,
    #[serde(default = "MethodCfg::default_clearance")]
    clearance: f32,
    #[serde(default = "MethodCfg::default_wire_radius")]
    wire_radius: f32,
    #[serde(default = "MethodCfg::default_epsilon")]
    epsilon: f32,
    #[serde(default = "MethodCfg::default_pre_shift")]
    pre_shift: bool,
    #[serde(default = "MethodCfg::default_iterations")]
    iterations: usize,
    // #[serde(default = "MethodCfg::default_inductive_force")]
    // inductance_force: f32,
    #[serde(default = "MethodCfg::default_radius_freedom")]
    radius_freedom: f32,
    // #[serde(default = "MethodCfg::default_radius_force")]
    // radius_force: f32,
    #[serde(default = "MethodCfg::default_center_freedom")]
    center_freedom: f32,
    // #[serde(default = "MethodCfg::default_center_force")]
    // center_force: f32,
    #[serde(default = "MethodCfg::default_verbose")]
    verbose: bool,
}
impl MethodCfg {
    pub fn default() -> Self {
        MethodCfg{
            circles: vec![CircleArgs::default()],
            clearance: Self::default_clearance(),
            wire_radius: Self::default_wire_radius(),
            epsilon: Self::default_epsilon(),
            pre_shift: Self::default_pre_shift(),
            iterations: Self::default_iterations(),
            // inductance_force: Self::default_inductive_force(),
            radius_freedom: Self::default_radius_freedom(),
            // radius_force: Self::default_radius_force(),
            center_freedom: Self::default_center_freedom(),
            // center_force: Self::default_center_force(),
            verbose: Self::default_verbose(),
        }
    }
    pub fn default_clearance() -> f32 {
        1.29
    }
    pub fn default_wire_radius() -> f32 {
        0.645
    }
    pub fn default_epsilon() -> f32 {
        0.15
    }
    pub fn default_pre_shift() -> bool {
        true
    }
    pub fn default_verbose() -> bool {
        false
    }
    pub fn default_iterations() -> usize {
        1
    }
    pub fn default_radius_freedom() -> f32 {
        0.2
    }
    pub fn default_center_freedom() -> f32 {
        0.5
    }
}

/// Single element arguments
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
struct CircleArgs {
    center: Point,
    #[serde(default = "CircleArgs::default_coil_radius", alias = "radius")]
    coil_radius: f32,
}
impl CircleArgs {
    pub fn default() -> Self {
        CircleArgs{
            coil_radius: Self::default_coil_radius(),
            center: Self::default_center(),
        }
    }
    pub fn default_coil_radius() -> f32 {
        5.0
    }
    pub fn default_center() -> Point {
        Point::new(0.0, 0.0, 0.0)
    }
}

impl methods::LayoutMethod for Method {
    /// Get the name of the layout method.
    fn get_method_name(&self) -> String {
        "Iterative Circles".to_string()
    }

    /// Parse the layout method config file
    fn parse_method_cfg(&mut self, method_cfg_file: &str) -> args::ProcResult<()>{
        let f = crate::io::open(method_cfg_file)?;
        self.method_args = serde_yaml::from_reader(f)?;
        Ok(())
    }

    fn do_layout(&self, surface: &Surface) -> layout::ProcResult<layout::Layout> {

        let mut layout_out = self.single_pass(surface, &self.method_args.circles)?;
        let mut new_circles = self.method_args.circles.clone();

        let iterations = self.method_args.iterations;

        for (i, _) in (0..iterations).enumerate() {
            println!("Iteration {}/{}...", (i + 1), self.method_args.iterations);
            if self.method_args.verbose {

                println!("Coupling factor estimates:");
                for (coil_id, coil) in layout_out.coils.iter().enumerate() {
                    for (other_coil_id, other_coil) in layout_out.coils.iter().enumerate() {
                        if coil_id < other_coil_id {
                            let coupling = coil.coupling_factor(other_coil, 1.0);
                            println!("Coil {} to Coil {}: {}", coil_id, other_coil_id, coupling);
                        }
                    }
                }
                println!();
            }
            // Adjust the centers of the coils
            for (coil_id, coil) in layout_out.coils.iter().enumerate() {
                let mut center = coil.center;
                let mut delta = GeoVector::zero();
                for (other_id, other_coil) in layout_out.coils.iter().enumerate() {
                    if coil_id != other_id {
                        let vector_between = center - other_coil.center;
                        let distance_scale = new_circles[coil_id].coil_radius + new_circles[other_id].coil_radius;
                        let d_rel = vector_between.norm() / distance_scale;
                        // TODO: This is all heuristic!
                        if d_rel < 1.1 {
                            let d_rel_err = -0.3 * coil.coupling_factor(other_coil, 1.0);
                            delta = delta - vector_between.normalize() * (d_rel_err * distance_scale) * (1.0 - (i as f32) / (iterations as f32) * 0.5);
                        } else if d_rel < 1.3 {
                            let d_rel_err = 0.1 * coil.coupling_factor(other_coil, 1.0);
                            delta = delta + vector_between.normalize() * (d_rel_err * distance_scale) * (1.0 - (i as f32) / (iterations as f32) * 0.5);
                        }
                    }
                }
                center = center + (delta.rej_onto(&coil.normal));
                new_circles[coil_id].center = center - (&center - surface);
            }         
            layout_out = self.single_pass(surface, &new_circles)?;
        }


        // Do inductance estimates
        if self.method_args.verbose {

            println!("Coupling factor estimates:");
            for (coil_id, coil) in layout_out.coils.iter().enumerate() {
                for (other_coil_id, other_coil) in layout_out.coils.iter().enumerate() {
                    if coil_id < other_coil_id {
                        let coupling = coil.coupling_factor(other_coil, 1.0);
                        println!("Coil {} to Coil {}: {}", coil_id, other_coil_id, coupling);
                    }
                }
            }
            println!();

            println!("Centers:");
            for (coil_id, coil) in layout_out.coils.iter().enumerate() {
                println!("Coil {}: {}", coil_id, coil.center);
            }
            println!();

            // println!("TESTING:");
            // let m = layout_out.coils[0].mutual_inductance(&layout_out.coils[1], 1.0);
            // let k = layout_out.coils[0].coupling_factor(&layout_out.coils[1], 1.0);
            // let d = (layout_out.coils[0].center - layout_out.coils[1].center).norm();
            // let dr = d / (new_circles[0].coil_radius + new_circles[1].coil_radius);
            // println!("{d}, {dr}, {k}, {m}");
        }
        
        Ok(layout_out)
    }
}

impl Method {

    /// Do a single pass of the iterative circles method
    fn single_pass(&self, surface: &Surface, circles: &Vec::<CircleArgs>) -> layout::ProcResult<layout::Layout> {
        let mut layout_out = layout::Layout::new();
        let verbose = self.method_args.verbose;

        // Iterate through the circles
        let wire_radius = self.method_args.wire_radius;
        let epsilon = self.method_args.epsilon;
        let pre_shift = self.method_args.pre_shift;

        for (coil_num, circle_args) in circles.iter().enumerate() {
            println!("Coil {}/{}...", (coil_num + 1), circles.len());
            
            // Grab arguments from the circle arguments
            let coil_radius = circle_args.coil_radius;
            
            // Snap the center to the surface
            let vec_to_surface = &circle_args.center - surface;
            let center = circle_args.center - vec_to_surface;

            // Create the circle through surface intersection with sphere
            let (cid, points, point_normals) = sphere_intersect(surface, center, coil_radius, epsilon);
            let coil_normal = surface.point_normals[cid];

            let coil = clean_coil_by_angle(
                center, coil_normal,
                coil_radius, wire_radius,
                points, point_normals,
                pre_shift, verbose
            )?;

            layout_out.coils.push(coil);
        }

        // Do overlaps
        self.mousehole_overlap(&mut layout_out, circles);

        if verbose { println!() };

        Ok(layout_out)
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

                let c = self.method_args.clearance + 2.0 * coil.wire_radius;
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
        for point in surface.points.iter() {
            for (i, circle) in circles.iter().enumerate() {
                let center = &circle.center;
                let radius = circle.coil_radius;
                if (point - center).norm() < radius {
                    for (j, other_circle) in circles.iter().enumerate() {
                        if i != j {
                            let other_center = &other_circle.center;
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
                            (coil.wire_radius + other_coil.wire_radius + self.method_args.clearance) * clearance_scale {
                            
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
        let s = serde_yaml::to_string(&method.method_args).unwrap();
        println!("{}", s);
    }
}
