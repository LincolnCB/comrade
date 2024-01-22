use crate::layout;
use layout::methods;
use layout::geo_3d::*;

use std::f32::consts::PI;

/// Iterative Circle Method struct.
/// This struct contains all the parameters for the Iterative Circle layout method.
#[derive(Debug)]
#[allow(dead_code)]
pub struct Method {
    /// Arguments for the Iterative Circle method.
    method_args: MethodArgs,
}

/// TODO: Expand, maybe use serde_yaml? Maybe try to write this as an external example?
#[derive(Debug)]
struct MethodArgs {
}

impl Method {
    pub fn new() -> crate::Result<Self> {
        Ok(Method{method_args: MethodArgs{}}) // TODO: Default values
    }
}

impl methods::LayoutMethod for Method {
    /// Get the name of the layout method.
    fn get_method_name(&self) -> String {
        "Iterative Circle".to_string()
    }

    /// Parse the layout method argument file
    #[allow(unused_variables)]
    fn parse_method_args(&mut self, arg_file: &str) {
        // TODO: Expand
    }

    /// Run the layout process with the given arguments.
    /// Uses the `layout` module.
    /// Takes parsed arguments (from `parse_layout_args` or future GUI).
    /// Returns a `Result` with the `layout::Layout` or an `Err`.
    fn do_layout(&self, surface: &Surface) -> crate::Result<layout::Layout> {
        let mut layout_out = layout::Layout::new();

        // TODO: Temporary hardcode coil size estimate
        let coil_area = surface.area / 4.0;
        let coil_radius = (coil_area / PI).sqrt();
        let epsilon : f32 = 0.299433 / 2.0;
        let section_count = 32;

        // for coil_n in 0..self.layout_args.coil_count {
        for coil_n in 0..1 {
            println!("Coil {}...", coil_n);

            let mut points = Vec::new();

            // TODO: Temporary hard code!
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

fn clean_by_angle(points: Vec<Point>, center: &Point, normal: &GeoVector, split_count: u32) -> crate::Result<Vec<Point>> {
    
    if split_count < 3 {
        return crate::err_string("Split count must be at least 3".to_string());
    }
    if points.len() < 3 {
        return crate::err_string("Not enough points to clean by angle".to_string());
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
            None => return crate::err_string("Math error: clean_by_angle, no minimum point found".to_string()),
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
            return crate::err_string("Math error: Angle bin out of range".to_string());
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
        return crate::err_string("Math error: Angle binning failed (no points within some bins)".to_string());
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
