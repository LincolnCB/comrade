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

            // TODO: Temporary hard code! 
            // Move this to deserialization after first deserialization works
            let center = Point::new(-1.305, 1.6107, 29.919);

            let mut cid = 0;
            let mut min_dist_to_center = surface.points[0].distance(&center);
            for (id, &point) in surface.points.iter().enumerate() {
                if (point.distance(&center) - coil_radius).abs() < epsilon {
                    points.push(point);

                    let n = surface.point_normals[id].normalize();

                    point_normals.push(n);
                }

                // Track the closest point to the sphere center
                if point.distance(&center) < min_dist_to_center {
                    min_dist_to_center = point.distance(&center);
                    cid = id;
                }
            }

            // Get coil normal from the closest point to the sphere center
            let normal = surface.point_normals[cid].normalize();

            println!("Uncleaned point count: {}", points.len());

            // let coil = clean_by_angle(
            //     center, normal,
            //     points, point_normals,
            //     section_count)?;
            let coil = clean_by_filter(
                center, normal, coil_radius,
                points, point_normals)?;

            println!("Cleaned point count: {}", coil.points.len());

            layout_out.coils.push(coil);
        }

        Ok(layout_out)
    }
}

/// Clean a set of points by filtering
#[allow(dead_code)]
fn clean_by_filter(
    center: Point,
    normal: GeoVector,
    radius: f32,
    mut points: Vec<Point>,
    point_normals: Vec<GeoVector>,
) -> layout::ProcResult<layout::Coil> {

    if points.len() < 3 {
        layout::err_str("Not enough points to clean by angle")?;
    }

    // Check that the point lists are the correct length
    if points.len() != point_normals.len() {
        layout::err_str(&format!("clean_by_filter: Point list (length: {0}) must be the same length as the normal list ({1})",
            points.len(), point_normals.len()))?;
    }

    let normal = normal.normalize();

    let mut angles = vec![[0.0, 0.0] as [Angle; 2]; points.len()];

    // TODO: Pre-shift points along radial tangent to very close to coil radius
    for (point_id, point) in points.iter_mut().enumerate() {
        let vec_to_point = (*point - center).normalize();
        let radial_tangent = vec_to_point.rej_onto(&point_normals[point_id]).normalize();
        let r_err = radius - point.distance(&center);

        let angle = radial_tangent.angle_to(&vec_to_point);
        if angle > PI / 1.5 {
            layout::err_str(&format!("Point {} {} is at too harsh an angle relative to the coil normal (centered at {}, normal {})",
                point_id, point, center, normal))?;
        }
        *point += r_err * radial_tangent / angle.cos();
    }
        

    // Calculate the angles
    // Get a reference zero-angle vector in the plane of the coil
    // Project the zhat vector onto the plane of the coil for this
    // If the normal is close to zhat, use the xhat vector instead
    let zhat = GeoVector::zhat();
    let zero_theta_vec = if normal.dot(&zhat).abs() < 0.999 {
        zhat.rej_onto(&normal).normalize()
    } else {
        GeoVector::xhat().rej_onto(&normal).normalize()
    };
    let pi2_theta_vec = zero_theta_vec.cross(&normal).normalize();
      
    // Convert each point to a pair of angles
    for (point_id, point) in points.iter().enumerate() {
        let vec_to_point = *point - center;
        let flat_vec = vec_to_point.rej_onto(&normal).normalize();

        angles[point_id][0] = zero_theta_vec.angle_to(&flat_vec);
        if flat_vec.cross(&zero_theta_vec).dot(&normal) < 0.0 {
            angles[point_id][0] = (2.0 * PI) - angles[point_id][0];
        }

        angles[point_id][1] = normal.angle_to(&vec_to_point);
    }

    angles.sort_by(|a, b| a[0].total_cmp(&b[0]));

    // TODO: Apply a filter to the angles

    // Reconstruct the coil
    let mut points = Vec::<Point>::new();

    for angle_pair in angles.iter() {
        let theta = angle_pair[0];
        let phi = angle_pair[1];

        let point = center + radius * (
                phi.sin() * (zero_theta_vec * theta.cos() + pi2_theta_vec * theta.sin())
                + normal * phi.cos()
            );
        
        points.push(point);
    }

    Ok(layout::Coil::new(center, normal, points)?)
}

/// Clean a set of points by angle.
#[allow(dead_code)]
fn clean_by_angle(
    center: Point,
    normal: GeoVector,
    points: Vec<Point>,
    point_normals: Vec<GeoVector>,
    split_count: u32,
) -> layout::ProcResult<layout::Coil> {
    
    if split_count < 3 {
        layout::err_str("Split count must be at least 3")?;
    }
    if points.len() < 3 {
        layout::err_str("Not enough points to clean by angle")?;
    }

    // Check that the point lists are the correct length
    if points.len() != point_normals.len() {
        layout::err_str(&format!("clean_by_angle: Point list (length: {0}) must be the same length as the normal list ({1})",
            points.len(), point_normals.len()))?;
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
        let flattened_vector = vector_to_point.rej_onto(&normal).normalize();
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

    for id in binned_points.iter() {
        let point_id = id.unwrap();
        out_points.push(points[point_id]);
        out_normals.push(point_normals[point_id]);
    }

    // Construct and output the coil
    Ok(layout::Coil::new(center, normal, out_points)?)
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
