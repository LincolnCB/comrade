use crate::layout;
use crate::geo_3d::*;
use layout::methods;
use methods::helper::{sphere_intersect, clean_coil_by_angle};

use serde::{Serialize, Deserialize};

/// Single Circle Method struct.
/// This struct contains all the parameters for the Single Circle layout method.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Method {
    center: Point,
    #[serde(default = "Method::default_coil_radius", alias = "radius")]
    coil_radius: f32,
    #[serde(default = "Method::default_wire_radius")]
    wire_radius: f32,
    #[serde(default = "Method::default_epsilon")]
    epsilon: f32,
    #[serde(default = "Method::default_pre_shift")]
    pre_shift: bool,
}
impl Method {
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
impl Default for Method {
    fn default() -> Self {
        Method{
            center: Method::default_center(),
            coil_radius: Method::default_coil_radius(),
            wire_radius: Method::default_wire_radius(),
            epsilon: Method::default_epsilon(),
            pre_shift: Method::default_pre_shift(),
        }
    }
}

impl methods::LayoutMethodTrait for Method {
    /// Get the name of the layout method.
    fn get_method_name(&self) -> &'static str {
        "Single Circle"
    }

    /// Run the layout process with the given arguments.
    /// Uses the `layout` module.
    /// Takes parsed arguments (from `parse_layout_args` or future GUI).
    /// Returns a `ProcResult` with the `layout::Layout` or an `Err`.
    fn do_layout(&self, surface: &Surface) -> layout::ProcResult<layout::Layout> {
        let mut layout_out = layout::Layout::new();

        println!("Coil 1/1...");

        // Intersect the surface with a sphere
        let (cid, points, point_normals) = 
            sphere_intersect(surface, self.center, self.coil_radius, self.epsilon);
        
        let coil_normal = surface.vertices[cid].normal.normalize();

        println!("Uncleaned point count: {}", points.len());

        let coil = clean_coil_by_angle(
            self.center, coil_normal,
            self.coil_radius, self.wire_radius,
            points, point_normals,
            self.pre_shift, true,
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
        let s = serde_yaml::to_string(method).unwrap();
        println!("{}", s);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
