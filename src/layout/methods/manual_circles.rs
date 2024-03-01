/*!
*   Manual Circles Method
*
*
!*/

use crate::{
    layout,
    args
};
use layout::methods;
use layout::geo_3d::*;
use methods::helper::{sphere_intersect, clean_coil_by_angle};

use serde::{Serialize, Deserialize};

/// Manual Circles Method struct.
/// This struct contains all the parameters for the Manual Circles layout method.
#[derive(Debug)]
pub struct Method {
    /// Arguments for the Manual Circles method.
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
    #[serde(default = "MethodCfg::default_verbose")]
    verbose: bool,
}
impl MethodCfg {
    pub fn default() -> Self {
        MethodCfg{
            circles: vec![CircleArgs::default()],
            clearance: Self::default_clearance(),
            epsilon: Self::default_epsilon(),
            wire_radius: Self::default_wire_radius(),
            pre_shift: Self::default_pre_shift(),
            verbose: Self::default_verbose(),
        }
    }
    pub fn default_clearance() -> f32 {
        1.29
    }
    pub fn default_verbose() -> bool {
        false
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
}

/// Single element arguments
#[derive(Debug, Serialize, Deserialize)]
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
        "Manual Circles".to_string()
    }

    /// Parse the layout method config file
    fn parse_method_cfg(&mut self, method_cfg_file: &str) -> args::ProcResult<()>{
        let f = crate::io::open(method_cfg_file)?;
        self.method_args = serde_yaml::from_reader(f)?;
        Ok(())
    }

    fn do_layout(&self, surface: &Surface) -> layout::ProcResult<layout::Layout> {
        let mut layout_out = layout::Layout::new();
        let verbose = self.method_args.verbose;

        // Iterate through the circles
        let circles = &self.method_args.circles;
        let wire_radius = self.method_args.wire_radius;
        let epsilon = self.method_args.epsilon;
        let pre_shift = self.method_args.pre_shift;

        for (coil_num, circle_args) in circles.iter().enumerate() {
            println!("Coil {}/{}...", (coil_num + 1), circles.len());
            
            // Grab arguments from the circle arguments
            let coil_radius = circle_args.coil_radius;
            let center = circle_args.center;

            // Create the circle through surface intersection with sphere
            let (cid, points, point_normals) =
                sphere_intersect(surface, center, coil_radius, epsilon);
            let coil_normal = surface.point_normals[cid].normalize();

            if verbose { println!("Uncleaned point count: {}", points.len()) };

            let coil = clean_coil_by_angle(
                center, coil_normal,
                coil_radius, wire_radius,
                points, point_normals,
                pre_shift, verbose
            )?;
    
            if verbose { println!("Cleaned point count: {}", coil.vertices.len()) };
    
            layout_out.coils.push(coil);
        }

        // Do overlaps
        self.mousehole_overlap(&mut layout_out);

        // Do inductance estimates
        if verbose {
            for (coil_id, coil) in layout_out.coils.iter().enumerate() {
                println!("Coil {} self-inductance: {:.2} nH", coil_id, coil.self_inductance(1.0));
            }

            println!("Mutual inductance estimate:");
            for (coil_id, coil) in layout_out.coils.iter().enumerate() {
                for (other_coil_id, other_coil) in layout_out.coils.iter().enumerate() {
                    if coil_id < other_coil_id {
                        let inductance = coil.mutual_inductance(other_coil, 1.0);
                        println!("Coil {} to Coil {}: {:.2} nH", coil_id, other_coil_id, inductance);
                    }
                }
            }
        }
        
        Ok(layout_out)
    }
}

impl Method {

    /// Do overlaps between the coils
    fn mousehole_overlap(&self, layout_out: &mut layout::Layout) {
        let intersections = self.get_intersections(layout_out, 2.0);
        
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
                point_lengths[p] = point_lengths[p - 1] + (coil.vertices[p].point - coil.vertices[p - 1].point).mag();
            }
            let coil_length = point_lengths[coil.vertices.len() - 1] + (coil.vertices[0].point - coil.vertices[coil.vertices.len() - 1].point).mag();
    
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
            for other_id in coil_id+1..self.method_args.circles.len() {
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
                let other_center = self.method_args.circles[other_id].center;
                let distance_to_other_coil = |p: usize| -> f32 {
                    let point = coil.vertices[p].point;
                    let vec_to_center = point - other_center;
                    vec_to_center.mag()
                };
                let inside_other_coil = |p: usize| -> bool {
                    distance_to_other_coil(p) < self.method_args.circles[other_id].coil_radius
                };
                for segment in segments.iter_mut() {
                    let mut p_prev = segment.start;
                    let mut p = segment.start + 1 % coil.vertices.len();

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

            // Enum for which segment to use
            #[derive(PartialEq)]
            enum Which {
                First,
                Second,
            }
            
            // Closure for merging segments
            let merge_segments = |first_seg: &IntersectionSegment, second_seg: &IntersectionSegment| -> Option<IntersectionSegment> {
                
                let (which_for_start, which_for_end) = match (first_seg.end < first_seg.start, second_seg.end < second_seg.start) {
                    (true, true) => { // Both wrap
                        match (first_seg.start < second_seg.start, first_seg.end < second_seg.end) {
                            (true, true) => (Which::First, Which::Second),
                            (true, false) => (Which::First, Which::First),
                            (false, true) => (Which::Second, Which::Second),
                            (false, false) => (Which::Second, Which::First),
                        }
                    },
                    (true, false) => { // First wraps
                        if first_seg.start < second_seg.start {
                            (Which::First, Which::First)
                        }
                        else if first_seg.end > second_seg.end {
                            (Which::First, Which::First)
                        }
                        else if first_seg.end > second_seg.start {
                            (Which::First, Which::Second)
                        }
                        else if first_seg.start < second_seg.end {
                            (Which::Second, Which::First)
                        }
                        else {
                            return None; // No intersection
                        }
                    },
                    (false, true) => { // Second wraps
                        if second_seg.start < first_seg.start {
                            (Which::Second, Which::Second)
                        }
                        else if second_seg.end > first_seg.end {
                            (Which::Second, Which::Second)
                        }
                        else if second_seg.end > first_seg.start {
                            (Which::Second, Which::First)
                        }
                        else if second_seg.start < first_seg.end {
                            (Which::First, Which::Second)
                        }
                        else {
                            return None; // No intersection
                        }
                    },
                    (false, false) => { // Neither wrap
                        if first_seg.start < second_seg.start {
                            if first_seg.end < second_seg.start {
                                return None; // No intersection
                            }
                            else if first_seg.end < second_seg.end {
                                (Which::First, Which::Second)
                            }
                            else {
                                (Which::First, Which::First)
                            }
                        }
                        else {
                            if second_seg.end < first_seg.start {
                                return None; // No intersection
                            }
                            else if second_seg.end < first_seg.end {
                                (Which::Second, Which::First)
                            }
                            else {
                                (Which::Second, Which::Second)
                            }
                        }
                    },
                };

                let start_segment = match which_for_start {
                    Which::First => first_seg,
                    Which::Second => second_seg,
                };
                let end_segment = match which_for_end {
                    Which::First => first_seg,
                    Which::Second => second_seg,
                };

                let start = start_segment.start;
                let end = end_segment.end;

                let length = padded_segment_length(start, end);
                
                let mut wire_crossings = start_segment.wire_crossings.clone();
                let mut end_wire_crossings = end_segment.wire_crossings.clone();
                
                // Offset the end wire crossings by the overlapping length -- merge_length_offset accounts for padding!
                let length_offset = match which_for_start == which_for_end {
                    false => merge_length_offset(start_segment.start, end_segment.start),
                    true => {
                        let other_segment = match which_for_start {
                            Which::First => second_seg,
                            Which::Second => first_seg,
                        };
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
                if let Some(merged) = merge_segments(&current_segment, &seg) {
                    current_segment = merged;
                } else {
                    merged_segments.push(current_segment);
                    current_segment = seg;
                }
            }
            // Handle wrapping
            if merged_segments.len() > 0 {
                if let Some(merged) = merge_segments(&current_segment, &merged_segments[0]) {
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
    fn get_adjacency(&self, surface: &Surface) -> Vec<Vec<bool>> {
        let mut adjacency: Vec<Vec<bool>> = vec![vec![false; self.method_args.circles.len()]; self.method_args.circles.len()];
        for point in surface.points.iter() {
            for (i, circle) in self.method_args.circles.iter().enumerate() {
                let center = &circle.center;
                let radius = circle.coil_radius;
                if (point - center).mag() < radius {
                    for (j, other_circle) in self.method_args.circles.iter().enumerate() {
                        if i != j {
                            let other_center = &other_circle.center;
                            let other_radius = other_circle.coil_radius;
                            if (point - other_center).mag() < other_radius {
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
    fn get_intersections(&self, intersecting_layout: &layout::Layout, clearance_scale: f32) -> Vec<Vec<Vec<usize>>> {
        let mut intersections: Vec<Vec<Vec<usize>>> = vec![vec![vec![]; self.method_args.circles.len()]; self.method_args.circles.len()];
        for (i, coil) in intersecting_layout.coils.iter().enumerate() {
            for (j, other_coil) in intersecting_layout.coils.iter().enumerate() {
                if i != j {
                    for (k, vertex) in coil.vertices.iter().enumerate() {
                        if ((vertex.point - other_coil.center).mag() - self.method_args.circles[j].coil_radius).abs() < 
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
