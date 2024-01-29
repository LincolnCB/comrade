use crate::{
    layout,
    args
};
use layout::methods;
use layout::geo_3d::*;

use std::f32::consts::PI;

/// Single Circle Method struct.
/// This struct contains all the parameters for the Single Circle layout method.
#[derive(Debug)]
#[allow(dead_code)]
pub struct Method {
    /// Arguments for the Single Circle method.
    method_args: MethodArgs,
}

/// TODO: Deserialize from yaml cfg file
#[derive(Debug)]
struct MethodArgs {
    coil_radius: f32,
    epsilon: f32,
    section_count: u32,
}

impl Method {
    pub fn new() -> args::ProcResult<Self> {
        Ok(Method{method_args: MethodArgs{
            coil_radius: 5.0,
            epsilon: 0.15,
            section_count: 32,
        }})
    }
}

impl methods::LayoutMethod for Method {
    /// Get the name of the layout method.
    fn get_method_name(&self) -> String {
        "Single Circle".to_string()
    }

    /// Parse the layout method argument file
    #[allow(unused_variables)]
    fn parse_method_args(&mut self, arg_file: &str) -> args::ProcResult<()>{
        // TODO: Deserialize here
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

            let mut points = Vec::new();

            // TODO: Temporary hard code! 
            // Move this to deserialization after first deserialization works
            let center = Point::new(-1.305, 1.6107, 29.919);
            let normal = GeoVector::new(-0.041038, 0.062606, 0.997194);

            for point in surface.points.iter() {
                if (point.distance(&center) - coil_radius).abs() < epsilon {
                    points.push(point.dup());
                }
            }

            println!("Uncleaned point count: {}", points.len());

            points = clean_by_angle(points, &center, &normal, section_count)?;

            println!("Cleaned point count: {}", points.len());

            let coil = layout::Coil::new(points, center)?;
            layout_out.coils.push(coil);
        }

        Ok(layout_out)
    }
}

fn clean_by_angle(points: Vec<Point>, center: &Point, normal: &GeoVector, split_count: u32) -> layout::ProcResult<Vec<Point>> {
    
    if split_count < 3 {
        layout::err_str("Split count must be at least 3")?;
    }
    if points.len() < 3 {
        layout::err_str("Not enough points to clean by angle")?;
    }

    // Initialize the angle bins
    let angle_step: Angle = 2.0 * PI / split_count as Angle;
    let mut bin_error: Vec<Angle> = vec![angle_step; split_count as usize];
    let mut binned_points: Vec<Option<Point>> = Vec::with_capacity(split_count as usize);
    for _ in 0..split_count {
        binned_points.push(None);
    }

    let angle_to_normal = |point: &Point| {
        let angle = normal.angle_to(&GeoVector::new_from_points(center, point));
        (PI / 2.0 - angle.abs()).abs()
    };

    // Pick a starting zero-angle direction by finding the point most perpendicular to the normal
    let zero_angle_vector = GeoVector::new_from_points(
        center, 
        match points.iter().min_by(|a, b| {angle_to_normal(a).total_cmp(&angle_to_normal(b))}) {
            Some(point) => point,
            None => layout::err_str("Math error: clean_by_angle, no minimum point found")?,
        }
    );

    // Iteratively bin the points
    for point in points.iter() {
        // Find the angle of the point relative to the zero-angle direction
        let vector_to_point = GeoVector::new_from_points(center, point);
        let flattened_vector = vector_to_point.proj_onto(&normal) - vector_to_point;
        let mut angle = zero_angle_vector.angle_to(&flattened_vector);

        // Check if the angle is in the correct direction
        if flattened_vector.cross(&zero_angle_vector).dot(normal) < 0.0 {
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
            binned_points[bin_id] = Some(point.dup());
        }

        // Optional debug: print the bins
        // tests::print_bins(&binned_points);
    }

    // Error if any bins are empty
    if binned_points.iter().any(|p| p.is_none()) {
        layout::err_str("Math error: Angle binning failed (no points within some bins)")?;
    }

    // Unwrap the points
    let mut out_points: Vec<Point> = binned_points.into_iter().map(|p| p.unwrap()).collect();

    // Construct the adjacency list
    let length = split_count as usize;

    out_points[0].adj = vec![length-1, 1];
    for (pid, point) in out_points.iter_mut().enumerate().skip(1).take(length-2) {
        point.adj = vec![pid-1, pid+1];
    }
    out_points[length-1].adj = vec![length-2, 0];

    Ok(out_points)
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
