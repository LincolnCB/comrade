use crate::{
    layout,
    mesh,
    args,
};
use mesh::methods;
use layout::geo_3d::*;

use serde::{Serialize, Deserialize};
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::LineWriter;

use std::f32::consts::PI;

/// GMSH Method struct.
/// This struct contains all the parameters for the GMSH meshing method.
#[derive(Debug)]
pub struct Method {
    /// Arguments for the GMSH method.
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
    #[serde(default = "MethodCfg::default_break_count", alias = "breaks")]
    break_count: usize,
    #[serde(default = "MethodCfg::default_angle_shift", alias = "angle")]
    angle_shift: f32,
    #[serde(default = "MethodCfg::default_lc")]
    lc: f32,
}
impl MethodCfg {
    pub fn default_break_count() -> usize {
        4
    }
    pub fn default_angle_shift() -> f32 {
        0.0
    }
    pub fn default_lc() -> f32 {
        0.010000
    }
    pub fn default() -> Self {
        MethodCfg{
            break_count: Self::default_break_count(),
            angle_shift: Self::default_angle_shift(),
            lc: Self::default_lc(),
        }
    }
}

/// GMSH Arc struct
struct Arc {
    start: usize,
    center: usize,
    end: usize,
}
/// GMSH Spline struct
struct Spline {
    points: Vec<usize>,
}

impl methods::MeshMethod for Method {
    /// Get the name of the meshing method.
    fn get_method_name(&self) -> String {
        "GMSH".to_string()
    }

    /// Get the output file extension for the meshing method.
    fn get_output_extension(&self) -> String {
        "geo".to_string()
    }

    /// Parse the meshing method config file
    fn parse_method_cfg(&mut self, method_cfg_file: &str) -> args::ProcResult<()>{
        let f = crate::io::open(method_cfg_file)?;
        self.method_args = serde_yaml::from_reader(f)?;
        Ok(())
    }

    /// Run the meshing process with the given arguments.
    /// Uses the `mesh` and `layout` modules.
    fn save_mesh(&self, layout: &layout::Layout, output_path: &str) -> mesh::ProcResult<()> {
        let output_path = output_path.to_string() + ".geo";

        let break_count = self.method_args.break_count;
        if break_count < 1 {
            mesh::err_str("Break count must be at least 1")?;
        }

        // Points
        let mut full_points = Vec::<Point>::new();

        let mut full_arcs = Vec::<Arc>::new();
        
        let mut full_splines = Vec::<Spline>::new();
        
        
        // Mesh each coil
        for (coil_n, coil) in layout.coils.iter().enumerate() {
            println!("Coil {}...", coil_n);

            let radius = coil.wire_radius;
            
            if coil.vertices.len() < break_count {
                mesh::err_str("Not enough points to clean by angle")?;
            }

            // Initialize the GMSH vectors
            let mut points = Vec::<Point>::new();
            let mut arcs = Vec::<Arc>::new();
            let mut splines = Vec::<Spline>::new();

            // Track offset for the full lists
            let points_offset = full_points.len();
            
            // Initialize the angle bins
            let angle_step: Angle = 2.0 * PI / break_count as Angle;
            let mut bin_error: Vec<Angle> = vec![angle_step; break_count as usize];
            let mut binned_points: Vec<Option<usize>> = vec![None as Option<usize>; break_count as usize];
            
            // Calculate the angles
            // Get a reference zero-angle vector in the plane of the coil
            // Project the zhat vector onto the plane of the coil for this
            // If the normal is close to zhat, use the xhat vector instead
            let normal = coil.normal;
            let zhat = GeoVector::zhat();
            let zero_theta_vec = if normal.dot(&zhat).abs() < 0.999 {
                zhat.rej_onto(&normal).normalize()
            } else {
                GeoVector::xhat().rej_onto(&normal).normalize()
            };
            
            // Convert each point to an angle and bin them
            let center = coil.center;
            for (point_id, vertex) in coil.vertices.iter().enumerate() {
                let point = &vertex.point;

                // Get the relevant vectors
                let vec_to_point = *point - center;
                let up_vec = vertex.wire_radius_normal;
                let out_vec = vec_to_point.rej_onto(&up_vec).normalize();
                
                // Add the four wire spline points to the list
                for i in 0..4 {
                    let theta = i as f32 * PI / 2.0;
                    let point = vertex.point + up_vec * radius * theta.cos() + out_vec * radius * theta.sin();
                    points.push(point);
                    full_points.push(point);
                }
                // Add the wire point to the list (some may be unused)
                points.push(vertex.point);
                full_points.push(vertex.point);
                
                let mut angle = zero_theta_vec.angle_to(&out_vec);

                if out_vec.cross(&zero_theta_vec).dot(&normal) < 0.0 {
                    angle = (2.0 * PI) - angle;
                }

                // Bin the point
                let bin_id = (angle / angle_step) as usize;
                if bin_id >= break_count as usize {
                    mesh::err_str("Math error: Angle bin out of range")?;
                }
                let error = (angle - bin_id as Angle * angle_step).abs();
                if error < bin_error[bin_id] {
                    bin_error[bin_id] = error;
                    binned_points[bin_id] = Some(point_id);
                }
            }

            // Error if any bins are empty
            if binned_points.iter().any(|p| p.is_none()) {
                mesh::err_str("Math error: Angle binning failed (no points within some bins)")?;
            }

            // Add the arcs
            for id in binned_points.iter() {
                let point_id = id.unwrap();

                // Add the four arcs
                for i in 0..4 {
                    let start = point_id * 5 + i;
                    let center = point_id * 5 + 4;
                    let end = point_id * 5 + (i + 1) % 4;
                    arcs.push(Arc{start, center, end});

                    // Add the arc to the full list
                    full_arcs.push(Arc{start: start + points_offset, center: center + points_offset, end: end + points_offset});
                }
            }

            // Add the splines
            let mut prev_id = binned_points[0].unwrap();
            for id in binned_points.iter().skip(1) {
                let point_id = id.unwrap();
                for i in 0..4 {
                    let mut spline_points = Vec::<usize>::new();
                    let mut spline_points_full = Vec::<usize>::new();
                    for j in prev_id..=point_id {
                        spline_points.push(j * 5 + i);
                        spline_points_full.push(j * 5 + i + points_offset);
                    }
                    splines.push(Spline{points: spline_points});
                    full_splines.push(Spline{points: spline_points_full});
                }
                prev_id = point_id;
            }
            // Get the last (possibly wrap-around) spline
            let first_id = binned_points[0].unwrap();
            for i in 0..4 {
                let mut spline_points = Vec::<usize>::new();
                let mut spline_points_full = Vec::<usize>::new();
                if first_id < prev_id {
                    for j in prev_id..coil.vertices.len() {
                        spline_points.push(j * 5 + i);
                        spline_points_full.push(j * 5 + i + points_offset);
                    }
                    for j in 0..=first_id {
                        spline_points.push(j * 5 + i);
                        spline_points_full.push(j * 5 + i + points_offset);
                    }
                } else {
                    for j in prev_id..=first_id {
                        spline_points.push(j * 5 + i);
                        spline_points_full.push(j * 5 + i + points_offset);
                    }
                }
                splines.push(Spline{points: spline_points});
                full_splines.push(Spline{points: spline_points_full});
            }

            // Save each coil to a separate file
            let numbered_output_path = output_path.replace(".geo", &format!("_c{}.geo", coil_n));
            println!("Saving coil {} to {}...", coil_n, numbered_output_path);
            match self.save_geo(&points, &arcs, &splines, &numbered_output_path) {
                Ok(_) => (),
                Err(error) => {
                    return Err(crate::io::IoError{file: output_path.to_string(), cause: error}.into());
                },
            };
        }

        // Save a full set of coils (often just for visualization)
        println!("Saving full array to {}", output_path);
        match self.save_geo(&full_points, &full_arcs, &full_splines, &output_path) {
            Ok(_) => (),
            Err(error) => {
                return Err(crate::io::IoError{file: output_path.to_string(), cause: error}.into());
            },
        };

        Ok(())
    }
}

impl Method {
    /// Save a GMSH .geo file
    fn save_geo(&self, points: &Vec<Point>, arcs: &Vec<Arc>, splines: &Vec<Spline>, output_path: &str) -> std::io::Result<()> {
        let file = OpenOptions::new().write(true).create(true).open(&output_path)?;
        let break_count = self.method_args.break_count;
        let spline_offset = arcs.len();

        let mut file = LineWriter::new(file);

        // Write the lc
        writeln!(file, "lc = {};", self.method_args.lc)?;
        writeln!(file)?;

        // Write the points
        for (point_id, point) in points.iter().enumerate() {
            writeln!(file, "Point({}) = {{{}, {}, {}, lc}};", point_id, point.x, point.y, point.z)?;
        }
        writeln!(file)?;


        // Write the arcs
        for (arc_id, arc) in arcs.iter().enumerate() {
            writeln!(file, "Circle({}) = {{{}, {}, {}}};", arc_id, arc.start, arc.center, arc.end)?;
        }
        writeln!(file)?;

        // Write the splines
        for (spline_n, spline) in splines.iter().enumerate() {
            let spline_id = spline_n + spline_offset;
            let mut spline_str = format!("Spline({}) = {{", spline_id);
            for (point_id, point) in spline.points.iter().enumerate() {
                spline_str.push_str(&point.to_string());
                if point_id < spline.points.len() - 1 {
                    spline_str.push_str(", ");
                }
            }
            spline_str.push_str("};");
            writeln!(file, "{}", spline_str)?;
        }
        writeln!(file)?;

        // Write the line loops
        let coil_count = arcs.len() / (4 * break_count);
        for coil_n in 0..coil_count {
            for segment_n in 0..break_count {
                for i in 0..4 {
                    let first_arc_id = coil_n * break_count * 4 + segment_n * 4 + i;
                    let second_arc_id = coil_n * break_count * 4 + ((segment_n + 1) % break_count) * 4 + i;
                    
                    let first_spline_id = first_arc_id + spline_offset;
                    let second_spline_id = coil_n * break_count * 4 + segment_n * 4 + (i + 1) % 4 + spline_offset;

                    let loop_id = first_arc_id;
                    writeln!(file, "Line Loop({}) = {{-{}, {}, {}, -{}}};", loop_id, first_arc_id, first_spline_id, second_arc_id, second_spline_id)?;
                }
            }
        }
        writeln!(file)?;

        // Write the ruled surfaces
        for id in 0..(coil_count * break_count * 4) {
            writeln!(file, "Ruled Surface({id}) = {{{id}}};")?;
        }
        writeln!(file)?;

        // Write the physical lines (four arcs)
        for (line_id, _) in arcs.iter().step_by(4).enumerate() {
            writeln!(file, "Physical Line({}) = {{{}, {}, {}, {}}};", line_id, line_id * 4, line_id * 4 + 1, line_id * 4 + 2, line_id * 4 + 3)?;
        }
        writeln!(file)?;

        // Write the physical surfaces (one coil)
        for coil_n in 0..coil_count {
            let mut surface_str = format!("Physical Surface({}) = {{", coil_n);
            for surface_id in coil_n * break_count * 4..(coil_n + 1) * break_count * 4 {
                surface_str.push_str(&surface_id.to_string());
                if surface_id < (coil_n + 1) * break_count * 4 - 1 {
                    surface_str.push_str(", ");
                }
            }
            surface_str.push_str("};");
            writeln!(file, "{}", surface_str)?;
        }
        writeln!(file)?;

        writeln!(file, "Coherence Mesh;")?;

        Ok(())
    }
}   
