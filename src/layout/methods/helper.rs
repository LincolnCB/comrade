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
pub fn bin_by_angle(
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

/// Clean a set of points by filtering
#[allow(dead_code)]
pub fn clean_by_angle(
    center: Point,
    normal: GeoVector,
    radius: f32,
    mut points: Vec<Point>,
    point_normals: Vec<GeoVector>,
    pre_shift: bool,
) -> layout::ProcResult<layout::Coil> {

    if points.len() < 3 {
        layout::err_str("Not enough points to clean by angle")?;
    }

    // Check that the point lists are the correct length
    if points.len() != point_normals.len() {
        layout::err_str(&format!("clean_by_angle: Point list (length: {0}) must be the same length as the normal list ({1})",
            points.len(), point_normals.len()))?;
    }

    let normal = normal.normalize();

    let mut angles = vec![[0.0, 0.0] as [Angle; 2]; points.len()];

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

    // TODO: Edge detection and reordering
    // TODO: Calculate theta from point selection epsilon
    // TODO: Make edge_buffer a variable
    // TODO: Handle edges that go around the start/end of the list
    let angle_eps = 0.005;
    let edge_buffer = 2;
    let mut in_edge = false;
    let mut prev_id = angles.len() - 1;
    let mut edge_start = angles.len() - 1;
    let mut edge_end;
    let mut edges = Vec::<(usize, usize)>::new();
    for (pid, angle_pair) in angles.iter().enumerate() {
        let prev_pair = &angles[prev_id];
        // let total_angle = (angle_pair[0] * angle_pair[0] + angle_pair[1] * angle_pair[1]).sqrt();

        if !in_edge {
            if (angle_pair[0] - prev_pair[0]).abs() < angle_eps && angle_pair[1] - prev_pair[1] > angle_eps {
                in_edge = true;
                edge_start = (prev_id + angles.len() - edge_buffer) % angles.len();
            }
        }
        else {
            if (angle_pair[0] - prev_pair[0]).abs() > angle_eps && angle_pair[1] - prev_pair[1] < angle_eps {
                in_edge = false;
                edge_end = (pid + edge_buffer) % angles.len();
                edges.push((edge_start, edge_end));
            }
        }

        prev_id = pid;
    }

    // Merge edges
    // TODO: Handle edges that go around the start/end of the list
    println!("Edges: {:?}", edges);
    let mut merged_edges = Vec::<(usize, usize)>::new();
    let mut edge = edges[0].clone();
    for i in 0..edges.len() {
        if i < edges.len() - 1 {
            let next_edge = edges[i + 1].clone();
            if edge.1 > next_edge.0 {
                edge.1 = next_edge.1;
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
    println!("Merged edges: {:?}", merged_edges);
    edges = merged_edges;
        
    // Reorder within the edges
    let anchor_buffer = 3;
    let mut i: usize = 0;
    let l1_angle = |a1: &[Angle; 2], a2: &[Angle; 2]| -> f32 {
        let dtheta = (a1[0] - a2[0]).abs();
        let dphi = (a1[1] - a2[1]).abs();
        dtheta + dphi
    };
    if edges.len() > 0 {
        let mut new_angles = Vec::<[Angle; 2]>::new();
        for edge in edges.iter() {
            let (start, end) = edge;
            let start = *start;
            let end = *end;
            let start_anchor = angles[if start > anchor_buffer { start - anchor_buffer } else { 0 }];
            // let end_anchor = angles[if end < angles.len() - 3 { end + 3 } else { angles.len() - 1 }];
            let mut sorted_edge = Vec::<[Angle; 2]>::new();
            for j in start..end {
                sorted_edge.push(angles[j]);
            }
            sorted_edge.sort_by(|a, b| l1_angle(&a, &start_anchor).total_cmp(&l1_angle(&b, &start_anchor)));

            if i < start {
                new_angles.extend_from_slice(&angles[i..start]);
            }

            new_angles.extend_from_slice(&sorted_edge);

            i = end;
        }

        if i < angles.len() {
            new_angles.extend_from_slice(&angles[i..angles.len()]);
        }
        assert_eq!(new_angles.len(), angles.len());
        angles = new_angles;
    }

    // TODO: Make smooth count a variable
    let smooth_count = 2;
    for _ in 0..smooth_count {
        for i in 0..angles.len() {
            let mut angle_pair = angles[i];
            let mut prev_angle_pair = if i > 0 { angles[i - 1] } else { angles[angles.len() - 1] };
            let mut next_angle_pair = if i < angles.len() - 1 { angles[i + 1] } else { angles[0] };
            
            if prev_angle_pair[0] - angle_pair[0] > PI {
                prev_angle_pair[0] -= 2.0 * PI;
            }
            if angle_pair[0] - prev_angle_pair[0] > PI {
                prev_angle_pair[0] += 2.0 * PI;
            }
    
            if next_angle_pair[0] - angle_pair[0] > PI {
                next_angle_pair[0] -= 2.0 * PI;
            }
            if angle_pair[0] - next_angle_pair[0] > PI {
                next_angle_pair[0] += 2.0 * PI;
            }
    
            angle_pair[0] = (angle_pair[0] + prev_angle_pair[0] + next_angle_pair[0]) / 3.0;
            angle_pair[1] = (angle_pair[1] + prev_angle_pair[1] + next_angle_pair[1]) / 3.0;
    
            angles[i] = angle_pair;
        } 
    }


    // Reconstruct the coil
    let mut points = Vec::<Point>::new();

    for (pid, angle_pair) in angles.iter().enumerate() {
        let theta = angle_pair[0];
        let phi = angle_pair[1];

        let point = center + radius * (
                phi.sin() * (zero_theta_vec * theta.cos() + pi2_theta_vec * theta.sin())
                + normal * phi.cos()
            );

        // NaN check
        if point.x.is_nan() || point.y.is_nan() || point.z.is_nan() {
            layout::err_str(&format!("BUG! helper::clean_by_angle \
                Point {} {} constructed as NaN (centered at {}, normal {}, angles [{}, {}]).",
                pid, point, center, normal, theta, phi))?;
        }
        
        points.push(point);
    }

    Ok(layout::Coil::new(center, normal, points)?)
}
