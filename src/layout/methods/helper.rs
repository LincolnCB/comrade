use crate::layout;
use crate::geo_3d::*;
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy)]
struct AngleFormat {
    theta: Angle,
    phi: Angle,
    point_id: usize,
}

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
    let mut min_dist_to_center = surface.vertices[0].point.distance(&center);

    // For each point in the surface
    for (point_id, surface_vertex) in surface.vertices.iter().enumerate() {
        let point = surface_vertex.point;
        // Calculate the distance from the center
        let distance = point.distance(&center);

        // If the distance is within epsilon of the radius
        if (radius - distance).abs() <= epsilon {
            // Add the point to the new points list
            new_points.push(point);

            // Add the point's normal to the new normals list
            new_normals.push(surface.vertices[point_id].normal.normalize());
        }

        // Track the closest point to the center
        if distance < min_dist_to_center {
            min_dist_to_center = distance;
            cid = point_id;
        }
    }

    if new_normals.iter().any(|n| n.has_nan()) {
        panic!("BUG! helper::sphere_intersect: NaN normal found in new_normals");
    }

    // Return the center id, new points, and new normals
    (cid, new_points, new_normals)
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
    verbose: bool,
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
            
            if (angle - PI / 2.0).abs() < (PI / 8.0) {
                continue;
            }
            
            let test_point = *point + r_err * radial_tangent / angle.cos();
            if test_point.x.is_nan() || test_point.y.is_nan() || test_point.z.is_nan() {
                panic!("BUG! Point {} {} shifted to NaN (centered at {}, normal {}, angle {}).",
                    point_id, point, center, normal, angle);
                }
                
                *point += r_err * radial_tangent / angle.cos();
        }
    } 
    
    // Calculate the angles
    // Get a reference zero-angle vector in the plane of the coil
    // Project the zhat vector onto the plane of the coil for this
    // If the normal is close to zhat, use the yhat vector instead
    let zhat = GeoVector::zhat();
    let zero_theta_vec = if normal.dot(&zhat).abs() < 0.999 {
        zhat.rej_onto(&normal).normalize()
    } else {
        GeoVector::yhat().rej_onto(&normal).normalize()
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
    if verbose { println!("Detecting edges...") };
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
        if verbose { println!("Merging edges...") };

        let mut merged_edges = Vec::<[usize; 2]>::new();
        let mut edge = edges[0].clone();
        for i in 0..edges.len() {
            if i < edges.len() - 1 {
                let next_edge = edges[i + 1].clone();
                if let Some((first_starts, first_ends)) = merge_segments(edge[0], edge[1], next_edge[0], next_edge[1]) {
                    edge[0] = if first_starts {edge[0]} else {next_edge[0]};
                    edge[1] = if first_ends {edge[1]} else {next_edge[1]};
                } else {
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
    // Handle the last edge -- merge if needed, and move it to the front if it wraps around
    if edges.len() > 1 {
        let first_edge = edges[0];
        let last_edge = edges[edges.len() - 1];

        if let Some((first_starts, first_ends)) = merge_segments(first_edge[0], first_edge[1], last_edge[0], last_edge[1]) {
            let new_edge = [if first_starts {first_edge[0]} else {last_edge[0]}, if first_ends {first_edge[1]} else {last_edge[1]}];
            edges[0] = new_edge;
            edges.pop();
        } else if last_edge[1] < last_edge[0] {
            edges.insert(0, last_edge);
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
            panic!("BUG! helper::clean_coil_by_angle \
                Point {} {} (originally point {}) \
                constructed as NaN (centered at {}, normal {}, angles [{}, {}]).",
                new_point_id, point, angle_pair.point_id, 
                center, normal, theta, phi);
        }
        
        points.push(point);
    }

    Ok(layout::Coil::new(center, normal, points, wire_radius, new_normals)?)
}

/// Add evenly distributed breaks to a coil by angle
#[allow(dead_code)]
pub fn add_even_breaks_by_angle(
    coil: &mut layout::Coil,
    break_count: usize,
    break_angle_offset: Angle,
    zero_angle_vec: GeoVector,
) -> layout::ProcResult<()> {
    let center = coil.center;
    let axis = coil.normal;
    let points = &coil.vertices.iter().map(|v| v.point).collect::<Vec<Point>>();

    let zero_angle_vec = zero_angle_vec.rej_onto(&axis).normalize();
    if zero_angle_vec.has_nan() {
        panic!("Math error: zero_angle_vec is NaN after rejection and normalizing");
    }
    let offset_zero_angle_vec = zero_angle_vec.rotate_around(&axis, break_angle_offset);

    let binned_points = bin_by_angle(points, break_count, center, axis, offset_zero_angle_vec)?;

    coil.breaks = Vec::<usize>::new();
    coil.port = Some(binned_points[0]);
    coil.breaks.extend(binned_points[1..].iter().cloned());

    Ok(())
}

/// Bin points by angle
pub fn bin_by_angle(points: &Vec::<Point>, bin_count: usize, center: Point, axis: GeoVector, zero_angle_vec: GeoVector) -> layout::ProcResult<Vec::<usize>> {

    // Initialize the angle bins
    let angle_step: Angle = (2.0 * PI) / bin_count as Angle;
    let mut bin_error: Vec<Angle> = vec![angle_step; bin_count as usize];
    let mut binned_points: Vec<Option<usize>> = vec![None as Option<usize>; bin_count as usize];

    let zero_angle_vec = zero_angle_vec.rej_onto(&axis).normalize();
    if zero_angle_vec.has_nan() {
        panic!("Math error: zero_angle_vec is NaN after rejection and normalizing");
    }

    // Iterate through points to bin
    for (point_id, point) in points.iter().enumerate() {
        if points.len() < bin_count {
            layout::err_str(&format!("Not enough points ({}) for that many breaks ({})", points.len(), bin_count))?;
        }
        
        // Calculate the angles

        // Get the relevant vectors
        let vec_to_point = *point - center;
        let out_vec = vec_to_point.rej_onto(&axis).normalize();
        
        let mut angle = zero_angle_vec.angle_to(&out_vec);

        if out_vec.cross(&zero_angle_vec).dot(&axis) < 0.0 && angle > 1e-6{
            angle = (2.0 * PI) - angle;
        }

        // Bin the point
        let bin_id = (angle / angle_step) as usize;
        if bin_id >= bin_count as usize {
            panic!("Math error: Angle ({angle}) bin {bin_id} out of range 0:{}", bin_count - 1);
        }
        let error = (angle - bin_id as Angle * angle_step).abs();
        if error < bin_error[bin_id] {
            bin_error[bin_id] = error;
            binned_points[bin_id] = Some(point_id);
        }
    }

    // Error if any bins are empty
    if binned_points.iter().any(|id| id.is_none()) {
        panic!("Math error: Angle binning (break count: {bin_count}) failed (no points within some bins)");
    }

    // Unwrap the points
    Ok(binned_points.iter().map(|id| id.unwrap()).collect())
}

/// Merge two segments of a coil
/// Returns whether the first segment is used for the start and the end, respectively
pub fn merge_segments(first_start: usize, first_end: usize, second_start: usize, second_end: usize) -> Option::<(bool, bool)> {

    Some(
        match (first_end < first_start, second_end < second_start) {
            (true, true) => { // Both wrap
                match (first_start <= second_start, first_end <= second_end) {
                    (true, true) => (true, false),
                    (true, false) => (true, true),
                    (false, true) => (false, false),
                    (false, false) => (false, true),
                }
            },
            (true, false) => { // First wraps
                if first_start <= second_start {
                    (true, true)
                }
                else if first_end >= second_end {
                    (true, true)
                }
                else if first_end >= second_start {
                    (true, false)
                }
                else if first_start <= second_end {
                    (false, true)
                }
                else {
                    return None; // No intersection
                }
            },
            (false, true) => { // Second wraps
                if second_start <= first_start {
                    (false, false)
                }
                else if second_end >= first_end {
                    (false, false)
                }
                else if second_end >= first_start {
                    (false, true)
                }
                else if second_start <= first_end {
                    (true, false)
                }
                else {
                    return None; // No intersection
                }
            },
            (false, false) => { // Neither wrap
                if first_start <= second_start {
                    if first_end < second_start {
                        return None; // No intersection
                    }
                    else if first_end <= second_end {
                        (true, false)
                    }
                    else {
                        (true, true)
                    }
                }
                else {
                    if second_end < first_start {
                        return None; // No intersection
                    }
                    else if second_end <= first_end {
                        (false, true)
                    }
                    else {
                        (false, false)
                    }
                }
            },
        }
    )
}

pub fn k_means(points: &Vec<Point>, k: usize, max_iter: usize, verbose: bool) -> Vec<Point> {
    let mut centers = Vec::<Point>::new();

    // Initialize the centers (no rng for now)
    centers.push(points[0]);
    for _ in 1..k {
        let mut max_dist = 0.0;
        let mut max_id = 0;
        for (point_id, point) in points.iter().enumerate() {
            let mut min_dist = point.distance(&centers[0]);
            for center in centers.iter() {
                let dist = point.distance(center);
                if dist < min_dist {
                    min_dist = dist;
                }
            }
            if min_dist > max_dist {
                max_dist = min_dist;
                max_id = point_id;
            }
        }
        centers.push(points[max_id]);
    }

    k_means_initialized(points, &centers, max_iter, verbose)
}

pub fn k_means_initialized(points: &Vec<Point>, starting_centers: &Vec<Point>, max_iter: usize, verbose: bool) -> Vec<Point> {
    // Clone initial points
    let mut centers = starting_centers.clone();
    let mut assignments = vec![0; points.len()];
    let k = centers.len();

    // Iterate through the max number of iterations
    for it in 0..max_iter {
        // Assign points to centers
        for (point_id, point) in points.iter().enumerate() {
            let mut min_dist = point.distance(&centers[0]);
            let mut min_center = 0;
            for (center_id, center) in centers.iter().enumerate() {
                let dist = point.distance(center);
                if dist < min_dist {
                    min_dist = dist;
                    min_center = center_id;
                }
            }
            assignments[point_id] = min_center;
        }

        // Update centers
        let mut new_centers = Vec::<Point>::new();
        for center_id in 0..k {
            let mut center_sum = GeoVector::zero();
            let mut count = 0;
            for (point_id, assignment) in assignments.iter().enumerate() {
                if *assignment == center_id {
                    center_sum += points[point_id].into();
                    count += 1;
                }
            }
            if count > 0 {
                center_sum /= count as f32;
            }
            new_centers.push(center_sum.into());
        }

        // Check for convergence
        let mut max_change = 0.0;
        for center_id in 0..k {
            let dist = centers[center_id].distance(&new_centers[center_id]);
            if dist > max_change {
                max_change = dist;
            }
        }

        if max_change < 1e-6 {
            if verbose {
                println!("K-means converged after {} iterations", it);
            }
            break;
        }

        centers = new_centers;

        if verbose && it % 10 == 0 {
            println!("K-means iteration {}: max change {}", it, max_change);
        }
    }

    centers
}

/// Get the closest point in a collection of points
pub fn closest_point<'a>(point: &Point, points: &'a Vec::<Point>) -> &'a Point {
    let mut closest = &points[0];
    let mut closest_distance = (point - closest).norm();
    for test_point in points.iter().skip(1) {
        let distance = (point - test_point).norm();
        if distance < closest_distance {
            closest = test_point;
            closest_distance = distance;
        }
    }
    closest
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
