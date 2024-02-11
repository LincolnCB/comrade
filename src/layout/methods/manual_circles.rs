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
use std::f32::consts::PI;

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
}
impl MethodCfg {
    pub fn default() -> Self {
        MethodCfg{
            circles: vec![CircleArgs::default()],
            clearance: Self::default_clearance(),
        }
    }
    pub fn default_clearance() -> f32 {
        2.0
    }
}

/// Single element arguments
#[derive(Debug, Serialize, Deserialize)]
struct CircleArgs {
    center: Point,
    #[serde(default = "CircleArgs::default_coil_radius", alias = "radius")]
    coil_radius: f32,
    #[serde(default = "CircleArgs::default_wire_radius")]
    wire_radius: f32,
    #[serde(default = "CircleArgs::default_epsilon")]
    epsilon: f32,
    #[serde(default = "CircleArgs::default_pre_shift")]
    pre_shift: bool,
}
impl CircleArgs {
    pub fn default() -> Self {
        CircleArgs{
            coil_radius: Self::default_coil_radius(),
            wire_radius: Self::default_wire_radius(),
            epsilon: Self::default_epsilon(),
            pre_shift: Self::default_pre_shift(),
            center: Self::default_center(),
        }
    }
    pub fn default_coil_radius() -> f32 {
        5.0
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

        // Iterate through the circles
        let circles = &self.method_args.circles;

        for (coil_num, circle_args) in circles.iter().enumerate() {
            println!("Coil {}/{}...", (coil_num + 1), circles.len());
            
            // Grab arguments from the circle arguments
            let coil_radius = circle_args.coil_radius;
            let wire_radius = circle_args.wire_radius;
            let epsilon = circle_args.epsilon;
            let pre_shift = circle_args.pre_shift;
            let center = circle_args.center;

            // Create the circle through surface intersection with sphere
            let (cid, points, point_normals) =
                sphere_intersect(surface, center, coil_radius, epsilon);
            let coil_normal = surface.point_normals[cid].normalize();

            println!("Uncleaned point count: {}", points.len());

            let coil = clean_coil_by_angle(
                center, coil_normal,
                coil_radius, wire_radius,
                points, point_normals,
                pre_shift,
            )?;
    
            println!("Cleaned point count: {}", coil.vertices.len());
    
            layout_out.coils.push(coil);
        }

        // Do mutual inductance estimate
        println!("Mutual inductance estimate:");
        for (coil_id, coil) in layout_out.coils.iter().enumerate() {
            for (other_coil_id, other_coil) in layout_out.coils.iter().enumerate() {
                if coil_id < other_coil_id {
                    let inductance = coil.mutual_inductance(other_coil, 1.0);
                    println!("Coil {} to Coil {}: {} uH", coil_id, other_coil_id, inductance);
                }
            }
        }

        let intersects = self.get_intersections(&layout_out);

        for (i, coil) in layout_out.coils.iter_mut().enumerate() {
            let mut intersecting_points = Vec::<usize>::new();

            // Get all the intersecting points in order, start segment management if there are any
            let mut any_intersections = false;
            for j in i+1..self.method_args.circles.len() {
                if intersects[i][j].len() > 0 {
                    any_intersections = true;
                    intersecting_points.extend_from_slice(&intersects[i][j]);
                }
            }
            if !any_intersections {
                continue;
            }

            // Group the intersecting points into segments
            intersecting_points.sort();
            intersecting_points.dedup();
            let mut segments = Vec::<Vec<(usize, f32)>>::new();

            let mut j_max = intersecting_points.len();
            let mut segment = Vec::<(usize, f32)>::new();

            // Check for a wraparound
            if intersecting_points[0] == 0 {
                let mut reverse = Vec::<(usize, f32)>::new();
                let mut j = 0;
                while intersecting_points[j_max - 1] == coil.vertices.len() - 1 - j {
                    reverse.push((intersecting_points[j_max - 1], 1.0));
                    j_max -= 1;
                    j += 1;

                    if j_max == 0 {
                        layout::err_str("Coil is entirely intersecting!")?;
                    }
                }
                
                for p in reverse.into_iter().rev() {
                    segment.push(p);
                }
            }

            // Define the segments
            for j in 0..j_max {
                let p = intersecting_points[j];
                segment.push((p, 0.0));
                if j < j_max - 1 {
                    if intersecting_points[j + 1] != p + 1 {
                        segments.push(segment);
                        segment = Vec::<(usize, f32)>::new();
                    }
                }
            }
            segments.push(segment);

            // Approximate the segment lengths
            for segment in segments.iter_mut() {
                let mut length = 0.0;
                let mut p = (segment[0].0 + coil.vertices.len() - 1) % coil.vertices.len();
                for i in 0..segment.len() {
                    let next_p = segment[i].0;
                    let dp = (coil.vertices[next_p].point - coil.vertices[p].point).mag();
                    length += dp;
                    segment[i].1 = length;
                    p = next_p;
                }
                let next_p = (segment[segment.len() - 1].0 + 1) % coil.vertices.len();
                let dp = (coil.vertices[next_p].point - coil.vertices[p].point).mag();
                length += dp;

                let scale_2 = self.method_args.clearance / 2.0 + coil.wire_radius;
                // The amount to offset the wire
                let offset = |l: f32| -> f32 {
                    let l_ratio = l / length;
                    if l_ratio < 0.25 {
                        scale_2 * (1.0 - (1.0 - 16.0 * l_ratio * l_ratio).sqrt())
                    }
                    else if l_ratio < 0.75 {
                        scale_2 * (1.0 + (1.0 - 16.0 * (l_ratio - 0.5) * (l_ratio - 0.5)).sqrt())
                    }
                    else {
                        scale_2 * (1.0 - (1.0 - 16.0 * (l_ratio - 1.0) * (l_ratio - 1.0)).sqrt())
                    }
                };
                // The amount to curve the wire
                let wire_rotation = |l: f32| -> f32 {
                    let l_ratio = l / length;
                    if l_ratio < 0.25 {
                        2.0 * PI * l_ratio
                    }
                    else if l_ratio < 0.75 {
                        2.0 * PI * (0.5 - l_ratio)
                    }
                    else {
                        2.0 * PI * (l_ratio - 1.0)
                    }
                };
                        
                for p in segment.iter_mut() {
                    coil.vertices[p.0].point = coil.vertices[p.0].point - coil.vertices[p.0].surface_normal * offset(p.1);
                    let surface_tangent = (coil.vertices[p.0].point - coil.center).rej_onto(&coil.vertices[p.0].surface_normal).normalize();
                    coil.vertices[p.0].wire_radius_normal = 
                        coil.vertices[p.0].wire_radius_normal
                        .rotate_around(&surface_tangent, wire_rotation(p.1));
                }
            }

                    
        }

        Ok(layout_out)
    }
}

impl Method {

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
    fn get_intersections(&self, intersecting_layout: &layout::Layout) -> Vec<Vec<Vec<usize>>> {
        let mut intersections: Vec<Vec<Vec<usize>>> = vec![vec![vec![]; self.method_args.circles.len()]; self.method_args.circles.len()];
        for (i, coil) in intersecting_layout.coils.iter().enumerate() {
            for (j, other_coil) in intersecting_layout.coils.iter().enumerate() {
                if i != j {
                    for (k, vertex) in coil.vertices.iter().enumerate() {
                        if ((vertex.point - other_coil.center).mag() - self.method_args.circles[j].coil_radius).abs() < 
                            coil.wire_radius + other_coil.wire_radius + self.method_args.clearance {
                            
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
