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
}
impl MethodCfg {
    pub fn default() -> Self {
        MethodCfg{
            circles: vec![CircleArgs::default()],
        }
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
    pub fn default() -> Self {
        CircleArgs{
            coil_radius: Self::default_coil_radius(),
            wire_radius: Self::default_wire_radius(),
            epsilon: Self::default_epsilon(),
            pre_shift: Self::default_pre_shift(),
            center: Self::default_center(),
        }
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

        println!("Mutual inductance estimate:");
        for (coil_id, coil) in layout_out.coils.iter().enumerate() {
            for (other_coil_id, other_coil) in layout_out.coils.iter().enumerate() {
                if coil_id < other_coil_id {
                    let inductance = coil.mutual_inductance(other_coil, 1.0);
                    println!("Coil {} to Coil {}: {} uH", coil_id, other_coil_id, inductance);
                }
            }
        }

        Ok(layout_out)
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
