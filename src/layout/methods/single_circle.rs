use crate::{
    layout,
    args
};
use layout::methods;
use layout::geo_3d::*;

use serde::{Serialize, Deserialize};
use std::f32::consts::PI;

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
    #[serde(default = "MethodArgs::default_radius", alias = "radius")]
    coil_radius: f32,
    #[serde(default = "MethodArgs::default_epsilon")]
    epsilon: f32,
    #[serde(default = "MethodArgs::default_section_count", alias = "sections")]
    section_count: u32,
}
impl MethodArgs {
    pub fn default_radius() -> f32 {
        5.0
    }
    pub fn default_epsilon() -> f32 {
        0.15
    }
    pub fn default_section_count() -> u32 {
        32
    }
    pub fn default() -> Self {
        MethodArgs{
            coil_radius: Self::default_radius(),
            epsilon: Self::default_epsilon(),
            section_count: Self::default_section_count(),
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
        let section_count = self.method_args.section_count;

        for coil_n in 0..1 {
            println!("Coil {}...", coil_n);

            let mut points = Vec::<Point>::new();
            let mut point_normals = Vec::<GeoVector>::new();
            let mut point_radial_tangents = Vec::<GeoVector>::new();

            // TODO: Temporary hard code! 
            // Move this to deserialization after first deserialization works
            let center = Point::new(-1.305, 1.6107, 29.919);
            let normal = GeoVector::new(-0.041038, 0.062606, 0.997194);

            for (id, &point) in surface.points.iter().enumerate() {
                if (point.distance(&center) - coil_radius).abs() < epsilon {
                    points.push(point);

                    let n = surface.point_normals[id].normalize();
                    let r = point - center;
                    let t = (r - r.proj_onto(&n)).normalize();

                    point_normals.push(n);
                    point_radial_tangents.push(t);
                }
            }

            println!("Uncleaned point count: {}", points.len());

            let coil = clean_by_angle(points, center, normal, point_normals, point_radial_tangents, section_count)?;

            println!("Cleaned point count: {}", coil.points.len());

            layout_out.coils.push(coil);
        }

        Ok(layout_out)
    }
}

/// Clean a set of points by angle.
fn clean_by_angle(
    points: Vec<Point>,
    center: Point,
    normal: GeoVector,
    point_normals: Vec<GeoVector>,
    point_radial_tangents: Vec<GeoVector>,
    split_count: u32,
) -> layout::ProcResult<layout::Coil> {
    
    if split_count < 3 {
        layout::err_str("Split count must be at least 3")?;
    }
    if points.len() < 3 {
        layout::err_str("Not enough points to clean by angle")?;
    }

    // Check that the point lists are the correct length
    if points.len() != point_normals.len() || points.len() != point_radial_tangents.len() {
        layout::err_str(&format!("clean_by_angle: Point list (length: {0}) must be the same length as the normal list ({1}) and radial tangent list ({2})!",
            points.len(), point_normals.len(), point_radial_tangents.len()))?;
    }

    // Initialize the angle bins
    let angle_step: Angle = 2.0 * PI / split_count as Angle;
    let mut bin_error: Vec<Angle> = vec![angle_step; split_count as usize];
    let mut binned_points: Vec<Option<usize>> = vec![None as Option<usize>; split_count as usize];

    let angle_to_normal = |point: &Point| {
        let angle = normal.angle_to(&(*point - center));
        (PI / 2.0 - angle.abs()).abs()
    };

    // Pick a starting zero-angle direction by finding the point most perpendicular to the normal
    let zero_angle_vector =         
        *match points.iter().min_by(|a, b| {angle_to_normal(a).total_cmp(&angle_to_normal(b))}) {
            Some(point) => point,
            None => layout::err_str("Math error: clean_by_angle, no minimum point found")?,
        } - center;

    // Iteratively bin the points
    for (point_id, point) in points.iter().enumerate() {
        // Find the angle of the point relative to the zero-angle direction
        let vector_to_point = *point - center;
        let flattened_vector = vector_to_point.proj_onto(&normal) - vector_to_point;
        let mut angle = zero_angle_vector.angle_to(&flattened_vector);

        // Check if the angle is in the correct direction
        if flattened_vector.cross(&zero_angle_vector).dot(&normal) < 0.0 {
            angle = (2.0 * PI) - angle;
        }

        // Bin the point
        let bin_id = (angle / angle_step) as usize;
        if bin_id >= split_count as usize {
            layout::err_str("Math error: Angle bin out of range")?;
        }
        let error = (angle - bin_id as Angle * angle_step).abs();
        if error < bin_error[bin_id] {
            bin_error[bin_id] = error;
            binned_points[bin_id] = Some(point_id);
        }

        // Optional debug: print the bins
        // tests::print_bins(&binned_points);
    }

    // Error if any bins are empty
    if binned_points.iter().any(|p| p.is_none()) {
        layout::err_str("Math error: Angle binning failed (no points within some bins)")?;
    }

    // Unwrap the points
    let mut out_points = Vec::<Point>::new();
    let mut out_normals = Vec::<GeoVector>::new();
    let mut out_radial_tangents = Vec::<GeoVector>::new();

    for id in binned_points.iter() {
        let point_id = id.unwrap();
        out_points.push(points[point_id]);
        out_normals.push(point_normals[point_id]);
        out_radial_tangents.push(point_radial_tangents[point_id]);
    }

    // Construct and output the coil
    Ok(layout::Coil::new(center, normal, out_points, out_normals, out_radial_tangents)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Optional print for visualization
    #[allow(dead_code)]
    pub fn print_bins(bins: &Vec<Option<Point>>) {
        print!("[");
        for bin in bins.iter() {
            match bin {
                Some(_) => print!("*"),
                None => print!("_"),
            }
        }
        println!("]");
    }
}
