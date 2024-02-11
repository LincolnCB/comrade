use crate::layout;
use layout::geo_3d::*;
use std::f32::consts::PI;

/// Find the points on a surface that intersect a sphere.
/// Returns the id of the point closest to the center, 
/// a vector of the intersected points, and the normals at those points.
pub fn sphere_intersect(
    surface: &Surface,
    center: Point,
    radius: f32,
    epsilon: f32,
) -> (usize, Vec::<Point>, Vec::<GeoVector>) {
    // Initialize the return values
    let mut new_points = Vec::<Point>::new();
    let mut new_normals = Vec::<GeoVector>::new();

    let mut cid = 0;
    let mut min_dist_to_center = surface.points[0].distance(&center);

    // For each point in the surface
    for (point_id, point) in surface.points.iter().enumerate() {
        // Calculate the distance from the center
        let distance = point.distance(&center);

        // If the distance is within epsilon of the radius
        if (radius - distance).abs() <= epsilon {
            // Add the point to the new points list
            new_points.push(*point);

            // Add the point's normal to the new normals list
            new_normals.push(surface.point_normals[point_id].normalize());
        }

        // Track the closest point to the center
        if distance < min_dist_to_center {
            min_dist_to_center = distance;
            cid = point_id;
        }
    }

    // Return the center id, new points, and new normals
    (cid, new_points, new_normals)
}

/// Clean a set of points by angle.
#[allow(dead_code)]
pub fn clean_coil_by_bins(
    center: Point,
    normal: GeoVector,
    wire_radius: f32,
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
        layout::err_str(&format!("clean_coil_by_angle: Point list (length: {0}) must be the same length as the normal list ({1})",
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
            None => layout::err_str("Math error: clean_coil_by_angle, no minimum point found")?,
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
    Ok(layout::Coil::new(center, normal, out_points, wire_radius, out_normals)?)
}

#[derive(Debug, Clone, Copy)]
struct AngleFormat {
    theta: Angle,
    phi: Angle,
    point_id: usize,
}

/// Clean a set of points by filtering
#[allow(dead_code)]
pub fn clean_coil_by_angle(
    center: Point,
    normal: GeoVector,
    radius: f32,
    wire_radius: f32,
    mut points: Vec<Point>,
    point_normals: Vec<GeoVector>,
    pre_shift: bool,
) -> layout::ProcResult<layout::Coil> {
    if points.len() < 3 {
        layout::err_str("Not enough points to clean by angle")?;
    }

    // Check that the point lists are the correct length
    if points.len() != point_normals.len() {
        layout::err_str(&format!("clean_coil_by_angle: Point list (length: {0}) must be the same length as the normal list ({1})",
            points.len(), point_normals.len()))?;
    }

    let normal = normal.normalize();

    
    // Shift points along the surface tangent to the right radius
    if pre_shift {
        for (point_id, point) in points.iter_mut().enumerate() {
            let vec_to_point = (*point - center).normalize();
            let radial_tangent = vec_to_point.rej_onto(&point_normals[point_id]).normalize();
            let r_err = radius - point.distance(&center);
            
            let angle = radial_tangent.angle_to(&vec_to_point);
            
            if (angle - PI / 2.0).abs() < 0.1 {
                continue;
                // layout::err_str(&format!("Point {} {} is at too harsh an angle relative to the coil normal \
                //     (centered at {}, normal {}). Try setting pre_shift to false.",
                //     point_id, point, center, normal))?;
            }
            
            let test_point = *point + r_err * radial_tangent / angle.cos();
            if test_point.x.is_nan() || test_point.y.is_nan() || test_point.z.is_nan() {
                layout::err_str(&format!("BUG! Point {} {} shifted to NaN (centered at {}, normal {}, angle {}).",
                    point_id, point, center, normal, angle))?;
                }
                
                *point += r_err * radial_tangent / angle.cos();
        }
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
    // Store the point_id for normals sorting
    let mut angles = Vec::<AngleFormat>::with_capacity(points.len());
    for (point_id, point) in points.iter().enumerate() {
        let mut angle_pair = AngleFormat {
            theta: 0.0,
            phi: 0.0,
            point_id,
        };

        let vec_to_point = *point - center;
        let flat_vec = vec_to_point.rej_onto(&normal).normalize();

        angle_pair.theta = zero_theta_vec.angle_to(&flat_vec);
        if flat_vec.cross(&zero_theta_vec).dot(&normal) < 0.0 {
            angle_pair.theta = (2.0 * PI) - angle_pair.theta;
        }

        angle_pair.phi = normal.angle_to(&vec_to_point);

        angles.push(angle_pair);
    }

    angles.sort_by(|a, b| a.theta.total_cmp(&b.theta));

    // Edge detection and reordering
    println!("Detecting edges...");
    // Check if sequential points are steeper than the angle ratio cap
    let angle_ratio_cap = 4.0;
    let is_past_ratio = |a1: &AngleFormat, a2: &AngleFormat| -> bool {
        let mut dtheta = (a1.theta - a2.theta).abs();
        if dtheta > PI {
            dtheta = 2.0 * PI - dtheta;
        }

        // Avoid division by zero
        if dtheta < 0.0001 {
            return true;
        }

        let dphi = (a1.phi - a2.phi).abs();
        dphi / dtheta > angle_ratio_cap
    };
    // TODO: Make edge_buffer a variable
    let edge_buffer = 2;
    if angles.len() < edge_buffer {
        layout::err_str(&format!("Edge buffer {edge_buffer} is larger than the number of points"))?;
    }
    let mut in_edge = false;
    let mut prev_id = angles.len() - 1;
    let mut edge_start = angles.len() - 1;
    let mut edge_end;
    let mut edges = Vec::<[usize; 2]>::new();
    for (pid, angle_pair) in angles.iter().enumerate() {
        let prev_pair = &angles[prev_id];

        if !in_edge {
            if is_past_ratio(angle_pair, prev_pair){
                in_edge = true;
                edge_start = (prev_id + angles.len() - edge_buffer) % angles.len();
            }
        }
        else {
            if !is_past_ratio(angle_pair, prev_pair) {
                in_edge = false;
                edge_end = (pid + edge_buffer) % angles.len();
                edges.push([edge_start, edge_end]);
            }
        }

        prev_id = pid;
    }
    // Close the last edge if necessary
    if in_edge {
        edge_end = edge_buffer - 1;
        edges.push([edge_start, edge_end]);
    }

    // Merge edges
    if edges.len() > 1 {
        println!("Merging edges...");
        let mut merged_edges = Vec::<[usize; 2]>::new();
        let mut edge = edges[0].clone();
        for i in 0..edges.len() {
            if i < edges.len() - 1 {
                let next_edge = edges[i + 1].clone();
                if edge[1] > next_edge[0] {
                    edge[1] = next_edge[1];
                    continue;
                }
                else {
                    merged_edges.push(edge);
                    edge = next_edge;
                }
            }
            else {
                merged_edges.push(edge);
            }
        }
        edges = merged_edges;
    }
    // Handle the case where the last edge wraps around
    if edges.len() > 0 {
        let first_edge = edges[0];
        let last_edge = edges[edges.len() - 1];
        if last_edge[1] < last_edge[0] {
            let unwrapped_last_end = last_edge[1] + angles.len();
            let unwrapped_first_start = 
                if first_edge[0] > first_edge[1] {
                    first_edge[0]
                }
                else {
                    first_edge[0] + angles.len()
                };
            if unwrapped_last_end > unwrapped_first_start {
                if edges.len() == 1 {
                    layout::err_str("Angle edge detection failed: one edge that fills the entire list.")?;
                }
                edges[0] = [last_edge[0], first_edge[1]];
            }
            else {
                edges.insert(0, last_edge);
            }
            edges.pop();
        }
    }

    // Reorder within the edges
    let anchor_buffer = 3;
    let mut i: usize = 0;
    let l1_angle = |a1: &AngleFormat, a2: &AngleFormat| -> f32 {
        let mut dtheta = (a1.theta - a2.theta).abs();
        if dtheta > PI {
            dtheta = 2.0 * PI - dtheta;
        }
        let dphi = (a1.phi - a2.phi).abs();
        dtheta + dphi
    };
    if edges.len() > 0 {
        let mut new_angles = Vec::<AngleFormat>::new();
        let mut end_wrap = Vec::<AngleFormat>::new();

        if edges[0][1] < edges[0][0] {
            // Handle the case where the first edge wraps around
            let mut wrapped_edge = Vec::<AngleFormat>::new();
            let anchor = angles[(edges[0][0] + angles.len() - anchor_buffer) % angles.len()];
            let wrap = angles.len() - edges[0][0];
            for j in edges[0][0]..angles.len() {
                wrapped_edge.push(angles[j]);
            }
            for j in 0..edges[0][1] {
                wrapped_edge.push(angles[j]);
            }

            wrapped_edge.sort_by(|a, b| l1_angle(&a, &anchor).total_cmp(&l1_angle(&b, &anchor)));
            
            new_angles.extend_from_slice(&wrapped_edge[wrap..wrapped_edge.len()]);
            end_wrap.extend_from_slice(&wrapped_edge[0..wrap]);
            i = edges[0][1];
        }

        for edge in edges.iter().skip(if edges[0][1] < edges[0][0] {1} else {0}) {
            let [start, end] = edge;
            let start = *start;
            let end = *end;
            let anchor = angles[(start + angles.len() - anchor_buffer) % angles.len()];
            let mut sorted_edge = Vec::<AngleFormat>::new();
            for j in start..end {
                sorted_edge.push(angles[j]);
            }
            sorted_edge.sort_by(|a, b| l1_angle(&a, &anchor).total_cmp(&l1_angle(&b, &anchor)));

            if i < start {
                new_angles.extend_from_slice(&angles[i..start]);
            }

            new_angles.extend_from_slice(&sorted_edge);

            i = end;
        }

        if i < (angles.len() - end_wrap.len()) {
            new_angles.extend_from_slice(&angles[i..(angles.len() - end_wrap.len())]);
        }
        new_angles.extend_from_slice(&end_wrap);
        assert_eq!(new_angles.len(), angles.len());
        angles = new_angles;
    }

    // Reorder the normals to match the points
    let mut new_normals = Vec::<GeoVector>::new();
    for angle_pair in angles.iter() {
        new_normals.push(point_normals[angle_pair.point_id]);
    }

    // Smooth the angles by averaging with neighbors
    // Smooth the normals as well
    // TODO: Make smooth count a variable
    let smooth_count = 8;
    for _ in 0..smooth_count {
        let mut prev_i = angles.len() - 1;
        let mut next_i = 1;
        for i in 0..angles.len() {
            // Grab the angles and normals
            let mut angle_pair = angles[i];
            let mut prev_angle_pair = angles[prev_i];
            let mut next_angle_pair = angles[next_i];

            let mut point_normal = new_normals[i];
            let prev_normal = new_normals[prev_i];
            let next_normal = new_normals[next_i];
            
            // Account for angles that wrap around
            if prev_angle_pair.theta - angle_pair.theta > PI {
                prev_angle_pair.theta -= 2.0 * PI;
            }
            if angle_pair.theta - prev_angle_pair.theta > PI {
                prev_angle_pair.theta += 2.0 * PI;
            }

            if next_angle_pair.theta - angle_pair.theta > PI {
                next_angle_pair.theta -= 2.0 * PI;
            }
            if angle_pair.theta - next_angle_pair.theta > PI {
                next_angle_pair.theta += 2.0 * PI;
            }
            
            // Average the angles and normals
            angle_pair.theta = (angle_pair.theta + prev_angle_pair.theta + next_angle_pair.theta) / 3.0;
            angle_pair.phi = (angle_pair.phi + prev_angle_pair.phi + next_angle_pair.phi) / 3.0;
            
            point_normal = (point_normal + prev_normal + next_normal).normalize();

            // Store the new angles and normals
            angles[i] = angle_pair;
            new_normals[i] = point_normal;

            // Update the indices
            prev_i = i;
            next_i = (i + 1) % angles.len();
        } 
    }


    // Reconstruct the coil
    let mut points = Vec::<Point>::new();

    for (new_point_id, angle_pair) in angles.iter().enumerate() {
        let theta = angle_pair.theta;
        let phi = angle_pair.phi;

        let point = center + radius * (
                phi.sin() * (zero_theta_vec * theta.cos() + pi2_theta_vec * theta.sin())
                + normal * phi.cos()
            );

        // NaN check
        if point.x.is_nan() || point.y.is_nan() || point.z.is_nan() {
            layout::err_str(&format!("BUG! helper::clean_coil_by_angle \
                Point {} {} (originally point {}) \
                constructed as NaN (centered at {}, normal {}, angles [{}, {}]).",
                new_point_id, point, angle_pair.point_id, 
                center, normal, theta, phi))?;
        }
        
        points.push(point);
    }

    Ok(layout::Coil::new(center, normal, points, wire_radius, new_normals)?)
}

mod debug {
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
