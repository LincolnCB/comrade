use crate::{
    layout,
    args
};
use layout::methods;
use layout::geo_3d::*;
use methods::helper::{sphere_intersect, clean_by_angle};

use serde::{Serialize, Deserialize};

/// Single Circle Method struct.
/// This struct contains all the parameters for the Single Circle layout method.
#[derive(Debug)]
pub struct Method {
    /// Arguments for the Single Circle method.
    method_args: MethodArgs,
}
impl Method {
    pub fn new() -> args::ProcResult<Self> {
        Ok(Method{method_args: MethodArgs::default()})
    }
}

/// Deserializer from yaml arg file
#[derive(Debug, Serialize, Deserialize)]
struct MethodArgs {
    center: Point,
    #[serde(default = "MethodArgs::default_radius", alias = "radius")]
    coil_radius: f32,
    #[serde(default = "MethodArgs::default_epsilon")]
    epsilon: f32,
    #[serde(default = "MethodArgs::default_pre_shift")]
    pre_shift: bool,
}
impl MethodArgs {
    pub fn default_radius() -> f32 {
        5.0
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
        MethodArgs{
            coil_radius: Self::default_radius(),
            epsilon: Self::default_epsilon(),
            pre_shift: Self::default_pre_shift(),
            center: Self::default_center(),
        }
    }
}

impl methods::LayoutMethod for Method {
    /// Get the name of the layout method.
    fn get_method_name(&self) -> String {
        "Single Circle".to_string()
    }

    /// Parse the layout method argument file
    fn parse_method_args(&mut self, arg_file: &str) -> args::ProcResult<()>{
        let f = crate::io::open(arg_file)?;
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
        let epsilon = self.method_args.epsilon;
        let pre_shift = self.method_args.pre_shift;
        let center = self.method_args.center;

        println!("Coil 1/1...");

        // Intersect the surface with a sphere
        let (cid, points, point_normals) = 
            sphere_intersect(surface, center, coil_radius, epsilon);
        
        let coil_normal = surface.point_normals[cid].normalize();

        println!("Uncleaned point count: {}", points.len());

        let coil = clean_by_angle(
            center, coil_normal, coil_radius,
            points, point_normals,
            pre_shift,
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
