use crate::{
    layout,
    args
};
use layout::methods;
use layout::geo_3d::*;
use methods::helper::{sphere_intersect, clean_coil_by_angle};

use serde::{Serialize, Deserialize};

/// Single Circle Method struct.
/// This struct contains all the parameters for the Single Circle layout method.
#[derive(Debug)]
pub struct Method {
    /// Arguments for the Single Circle method.
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
    center: Point,
    #[serde(default = "MethodCfg::default_coil_radius", alias = "radius")]
    coil_radius: f32,
    #[serde(default = "MethodCfg::default_wire_radius")]
    wire_radius: f32,
    #[serde(default = "MethodCfg::default_epsilon")]
    epsilon: f32,
    #[serde(default = "MethodCfg::default_pre_shift")]
    pre_shift: bool,
}
impl MethodCfg {
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
impl Default for MethodCfg {
    fn default() -> Self {
        MethodCfg{
            center: MethodCfg::default_center(),
            coil_radius: MethodCfg::default_coil_radius(),
            wire_radius: MethodCfg::default_wire_radius(),
            epsilon: MethodCfg::default_epsilon(),
            pre_shift: MethodCfg::default_pre_shift(),
        }
    }
}

impl methods::LayoutMethod for Method {
    /// Get the name of the layout method.
    fn get_method_name(&self) -> String {
        "Single Circle".to_string()
    }

    /// Parse the layout method config file
    fn parse_method_cfg(&mut self, method_cfg_file: &str) -> args::ProcResult<()>{
        let f = crate::io::open(method_cfg_file)?;
        self.method_args = serde_yaml::from_reader(f)?;
        Ok(())
    }

    /// Run the layout process with the given arguments.
    /// Uses the `layout` module.
    /// Takes parsed arguments (from `parse_layout_args` or future GUI).
    /// Returns a `ProcResult` with the `layout::Layout` or an `Err`.
    fn do_layout(&self, surface: &Surface) -> layout::ProcResult<layout::Layout> {
        let mut layout_out = layout::Layout::new();

        // Grab arguments to save typing
        let coil_radius = self.method_args.coil_radius;
        let wire_radius = self.method_args.wire_radius;
        let epsilon = self.method_args.epsilon;
        let pre_shift = self.method_args.pre_shift;
        let center = self.method_args.center;

        println!("Coil 1/1...");

        // Intersect the surface with a sphere
        let (cid, points, point_normals) = 
            sphere_intersect(surface, center, coil_radius, epsilon);
        
        let coil_normal = surface.vertices[cid].normal.normalize();

        println!("Uncleaned point count: {}", points.len());

        let coil = clean_coil_by_angle(
            center, coil_normal,
            coil_radius, wire_radius,
            points, point_normals,
            pre_shift, true,
        )?;

        println!("Cleaned point count: {}", coil.vertices.len());

        layout_out.coils.push(coil);

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

#[cfg(test)]
mod tests {
    use super::*;
}
