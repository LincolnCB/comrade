/*!
*   Manual Circles Method with Symmetry
*
*
!*/

use crate::layout;
use crate::geo_3d::*;
use layout::methods;
use methods::helper::{
    sphere_intersect,
    sphere_intersect_symmetric,
    clean_coil_by_angle,
    merge_segments,
    add_even_breaks_by_angle
};

use serde::{Serialize, Deserialize};
use itertools::concat;

/// Manual Circles Method struct.
/// This struct contains all the parameters for the Manual Circles layout method.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Method {
    circles: Vec<CircleArgs>,
    #[serde(default = "Method::default_symmetry_plane", alias = "plane")]
    symmetry_plane: Option<Plane>,
    #[serde(default = "Method::default_clearance")]
    clearance: f32,
    #[serde(default = "Method::default_wire_radius")]
    wire_radius: f32,
    #[serde(default = "Method::default_epsilon")]
    epsilon: f32,
    #[serde(default = "Method::default_pre_shift")]
    pre_shift: bool,
    #[serde(default = "Method::default_zero_angle_vector")]
    zero_angle_vector: GeoVector,
    #[serde(default = "Method::default_backup_zero_angle_vector")]
    backup_zero_angle_vector: GeoVector,
    #[serde(default = "Method::default_verbose")]
    verbose: bool,
}
impl Method {
    pub fn example_symmetry_plane() -> Option<Plane> {
        Some(Plane::from_normal_and_offset(GeoVector::xhat(), 0.0))
    }
    pub fn default_symmetry_plane() -> Option<Plane> {
        None
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
    pub fn default_zero_angle_vector() -> GeoVector {
        GeoVector::zhat()
    }
    pub fn default_backup_zero_angle_vector() -> GeoVector {
        GeoVector::yhat()
    }
}
impl Default for Method {
    fn default() -> Self {
        Method{
            circles: vec![CircleArgs::default(); 2],
            symmetry_plane: Self::example_symmetry_plane(),
            clearance: Self::default_clearance(),
            epsilon: Self::default_epsilon(),
            wire_radius: Self::default_wire_radius(),
            pre_shift: Self::default_pre_shift(),
            zero_angle_vector: Self::default_zero_angle_vector(),
            backup_zero_angle_vector: Self::default_backup_zero_angle_vector(),
            verbose: Self::default_verbose(),
        }
    }
}

/// Single element arguments
#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
struct CircleArgs {
    center: Point,
    #[serde(default = "CircleArgs::default_coil_radius", alias = "radius")]
    coil_radius: f32,
    #[serde(default = "CircleArgs::default_break_count", alias = "breaks")]
    break_count: usize,
    #[serde(default = "CircleArgs::default_break_angle_offset", alias = "angle")]
    break_angle_offset: f32,
    #[serde(default = "CircleArgs::default_on_symmetry_plane", alias = "on_sym")]
    on_symmetry_plane: bool,
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

impl methods::LayoutMethodTrait for Method {
    /// Get the name of the layout method.
    fn get_method_display_name(&self) -> &'static str {
        "Manual Circles"
    }

    fn do_layout(&self, surface: &Surface) -> layout::ProcResult<layout::Layout> {
        let mut layout_out = layout::Layout::new();
        let verbose = self.verbose;

        // Grab the circle arguments
        let wire_radius = self.wire_radius;
        let epsilon = self.epsilon;
        let pre_shift = self.pre_shift;

        // Store boundary points
        let boundary_points: Vec<Point> = if let Some(symmetry_plane) = &self.symmetry_plane {
            // Get the original boundary points, trimmed by the symmetry plane
            surface.get_boundary_vertex_indices().iter()
            // Get the point of the vertex
            .map(|v| surface.vertices[*v].point)
            // Filter out points that were trimmed by the symmetry plane
            .filter(|point| {
                let distance = symmetry_plane.distance_to_point(point);
                distance.abs() >= 1.0e-6
            }).collect()
        } else {
            surface.get_boundary_vertex_indices().iter()
            .map(|v| surface.vertices[*v].point).collect()
        };
                
        
        // Get the closest boundary point to a given point
        let closest_boundary_point = |point: &Point| -> Point {
            let mut closest = boundary_points[0];
            let mut closest_distance = (*point - closest).norm();
            for boundary_point in boundary_points.iter().skip(1) {
                let distance = (point - boundary_point).norm();
                if distance < closest_distance {
                    closest = *boundary_point;
                    closest_distance = distance;
                }
            }
            closest
        };

        // Shrink initial radii to keep the coils within the boundary
        for (coil_id, circle) in self.circles.iter().enumerate() {
            let boundary_point = closest_boundary_point(&circle.center);
            let vec_to_boundary = circle.center - boundary_point;
            let distance_to_boundary = vec_to_boundary.norm();
            if distance_to_boundary < circle.coil_radius {
                println!("WARNING: Coil {} too close to boundary, radius of {:.2} but distance of {:.2}", coil_id, circle.coil_radius, distance_to_boundary);
            }
        }
        
        // Extract the surface
        let surface = if let Some(symmetry_plane) = &self.symmetry_plane {
            // Replace the surface with the trimmed surface
            let (trimmed_surface, _) = surface.trim_by_plane(symmetry_plane, true);
            trimmed_surface
        } else {
            (*surface).clone()
        };

        let circles = if let Some(symmetry_plane) = &self.symmetry_plane {
            // Separate the coils by their symmetry
            let mut sym_circles = Vec::<CircleArgs>::new();
            let mut pos_circles = Vec::<CircleArgs>::new();
            let mut neg_circles = Vec::<CircleArgs>::new();
            for (circle_num, circle) in self.circles.iter().enumerate() {
                if circle.on_symmetry_plane {
                    // Make sure the circle is on the symmetry plane
                    let mut circle = circle.clone();
                    if symmetry_plane.distance_to_point(&circle.center).abs() > epsilon {
                        println!("WARNING: Circle {} more than epsilon ({}) from symmetry plane, moving to symmetry plane", circle_num, epsilon);
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
                    if symmetry_plane.distance_to_point(&circle.center).abs() < epsilon {
                        println!("WARNING: Circle {} close to symmetry plane, may cause issues", circle_num);
                    }
                    pos_circles.push(circle);

                    // Add the flipped circle
                    let mut neg_circle = circle.clone();
                    neg_circle.center = neg_circle.center.reflect_across(&symmetry_plane);
                    neg_circles.push(neg_circle);
                }
            }

            // Count the total number of circles
            let total_circle_count = sym_circles.len() + pos_circles.len() + neg_circles.len();

            // Create the coils for the on-symmetry circles
            for (i, circle_args) in sym_circles.iter().enumerate() {
                println!("Coil {}/{} [on symmetry plane]...", (i + 1), total_circle_count);
                
                // Grab arguments from the circle arguments
                let coil_radius = circle_args.coil_radius;
                let center = circle_args.center;

                // Create the circle through surface intersection with sphere
                let (cid, points, point_normals) =
                    sphere_intersect_symmetric(&surface, center, coil_radius, epsilon, &symmetry_plane);
                let coil_normal = surface.vertices[cid].normal.normalize();

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

            // Create the coils for the positive circles
            for (i, circle_args) in pos_circles.iter().enumerate() {
                println!("Coil {}/{} [positive side of symmetry plane]...", (i + sym_circles.len() + 1), total_circle_count);
                
                // Grab arguments from the circle arguments
                let coil_radius = circle_args.coil_radius;
                let center = circle_args.center;

                // Create the circle through surface intersection with sphere
                let (cid, points, point_normals) =
                    sphere_intersect_symmetric(&surface, center, coil_radius, epsilon, &symmetry_plane);
                let coil_normal = surface.vertices[cid].normal.normalize();

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

            // Create the coils for the flipped circles
            for i in 0..pos_circles.len() {
                println!("Coil {}/{} [negative side of symmetry plane]...", 
                    (i + sym_circles.len() + pos_circles.len() + 1), total_circle_count);
                let mut neg_coil = layout_out.coils[sym_circles.len() + i].clone();
                neg_coil.center = neg_coil.center.reflect_across(&symmetry_plane);
                neg_coil.normal = neg_coil.normal.reflect_across(&symmetry_plane.get_normal());

                for vertex in neg_coil.vertices.iter_mut() {
                    vertex.point = vertex.point.reflect_across(&symmetry_plane);
                    vertex.surface_normal = vertex.surface_normal.reflect_across(&symmetry_plane.get_normal());
                    vertex.wire_radius_normal = vertex.wire_radius_normal.reflect_across(&symmetry_plane.get_normal());
                    let temp = vertex.next_id;
                    vertex.next_id = vertex.prev_id;
                    vertex.prev_id = temp;
                }

                layout_out.coils.push(neg_coil);
            }

            // Collect all the circles
            concat(vec![sym_circles.clone(), pos_circles.clone(), neg_circles.clone()])
        } else {
            for (coil_num, circle_args) in self.circles.iter().enumerate() {
                println!("Coil {}/{}...", (coil_num + 1), self.circles.len());
                
                // Grab arguments from the circle arguments
                let coil_radius = circle_args.coil_radius;
                let center = circle_args.center;

                // Create the circle through surface intersection with sphere
                let (cid, points, point_normals) =
                    sphere_intersect(&surface, center, coil_radius, epsilon);
                let coil_normal = surface.vertices[cid].normal.normalize();

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
            self.circles.clone()
        };

        // Do overlaps
        self.mousehole_overlap(&mut layout_out, &circles);

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

        // Add breaks
        for (coil_id, coil) in layout_out.coils.iter_mut().enumerate() {
            let break_count = circles[coil_id].break_count;
            let break_angle_offset_rad = circles[coil_id].break_angle_offset * std::f32::consts::PI / 180.0;
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

    /// Do overlaps between the coils
    fn mousehole_overlap(&self, layout_out: &mut layout::Layout, circles: &Vec::<CircleArgs>) {
        let intersections = self.get_intersections(layout_out, 2.0, circles);
        
        // Structure for managing intersecting segments
        #[derive(Clone, Debug)]
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
                let other_intersections = &intersections[coil_id][other_id];

                // Ignore loops entirely contained within other loops
                if coil.vertices.len() - other_intersections.len() < 2 {
                    continue;
                }

                if other_intersections.len() > 0 {
                    any_intersections = true;
                    
                    let mut start = other_intersections[0];
                    let mut end;
                    
                    // Check for wraparound
                    let mut i_max = other_intersections.len();
                    if other_intersections[0] == 0 {
                        for (rev_id, p) in other_intersections.iter().rev().enumerate() {
                            if *p != coil.vertices.len() - 1 - rev_id {
                                i_max = other_intersections.len() - rev_id;
                                start = other_intersections[i_max % other_intersections.len()];
                                break;
                            }
                        } 
                    }

                    // Define the segments for this other coil
                    for i in 1..i_max {
                        let p = other_intersections[i];
                        let prev_p = other_intersections[i - 1];
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
                    end = other_intersections[i_max - 1];
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
