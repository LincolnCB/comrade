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
        0.002
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
#[derive(Clone)]
struct Arc {
    start: usize,
    center: usize,
    end: usize,
}
/// GMSH Spline struct
#[derive(Clone)]
struct Spline {
    points: Vec<usize>,
}
/// Collection of points, arcs, and splines for GMSH
#[derive(Clone)]
struct Loop {
    points: Vec<Point>,
    arcs: Vec<Arc>,
    splines: Vec<Spline>,
    self_inductance: f32,
}
impl Loop {
    pub fn new() -> Self {
        Loop{points: Vec::new(), arcs: Vec::new(), splines: Vec::new(), self_inductance: 0.0}
    }
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

        let mut full_loops = Vec::<Loop>::new();
        
        // Mesh each coil
        for (coil_n, coil) in layout.coils.iter().enumerate() {
            println!("Coil {}...", coil_n);

            let radius = coil.wire_radius;
            
            if coil.vertices.len() < break_count {
                mesh::err_str(&format!("Not enough points ({}) for that many breaks ({})", coil.vertices.len(), break_count))?;
            }

            // Initialize the GMSH vectors
            let mut single_loop = Loop::new();
            single_loop.self_inductance = coil.self_inductance(1.0);
            
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
            // Also add the radial points used in the splines, while we're iterating
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
                    single_loop.points.push(point);
                }
                // Add the wire point to the list (some may be unused)
                single_loop.points.push(vertex.point);
                
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
                mesh::err_str(&format!("Math error: Angle binning (break count: {break_count}) failed (no points within some bins)"))?;
            }

            // Add two capacitor breaks on either side of the first binned break (the port)
            // Position them at the first points in either direction from the port that are at least 2*lc away
            let port_id = binned_points[0].unwrap();

            // Upper side capacitor break:
            let mut upper_capacitor_break_id = port_id;
            let mut distance = 0.0;
            while distance < 2.0 * self.method_args.lc {
                let previously_checked_id = upper_capacitor_break_id;
                upper_capacitor_break_id = coil.vertices[upper_capacitor_break_id].next_id;
                distance += (coil.vertices[upper_capacitor_break_id].point - coil.vertices[previously_checked_id].point).norm();
                if upper_capacitor_break_id == binned_points[1].unwrap() {
                    mesh::err_str("Math error: Nearby capacitor break (positive idx direction) not found before first break")?;
                }
            }
            
            // Lower side capacitor break:
            let mut lower_capacitor_break_id = port_id;
            let mut distance = 0.0;
            while distance < 2.0 * self.method_args.lc {
                let previously_checked_id = lower_capacitor_break_id;
                lower_capacitor_break_id = coil.vertices[lower_capacitor_break_id].prev_id;
                distance += (coil.vertices[lower_capacitor_break_id].point - coil.vertices[previously_checked_id].point).norm();
                if lower_capacitor_break_id == binned_points[break_count - 1].unwrap() {
                    mesh::err_str("Math error: Nearby capacitor break (negative idx direction) not found before last break")?;
                }
            }

            let mut break_points = vec![port_id, upper_capacitor_break_id];
            break_points.extend(binned_points.iter().skip(1).map(|p| p.unwrap()));
            break_points.push(lower_capacitor_break_id);

            // Add the arcs
            for id in break_points.iter() {
                // Add the four arcs per point
                for i in 0..4 {
                    let start = id * 5 + i;
                    let center = id * 5 + 4;
                    let end = id * 5 + (i + 1) % 4;
                    single_loop.arcs.push(Arc{start, center, end});
                }
            }

            // Add the splines.
            for break_number in 0..break_points.len() {
                
                let id = break_points[break_number];
                let next_id = break_points[(break_number + 1) % break_points.len()];
                // Add the four splines per point
                for i in 0..4 {
                    let mut spline_points = Vec::<usize>::new();

                    // Handle potential wraparound
                    if next_id < id {
                        for j in id..coil.vertices.len() {
                            spline_points.push(j * 5 + i);
                        }
                        for j in 0..=next_id {
                            spline_points.push(j * 5 + i);
                        }
                    } else {
                        for j in id..=next_id {
                            spline_points.push(j * 5 + i);
                        }
                    }
                    single_loop.splines.push(Spline{points: spline_points});
                }
            }

            // Save each coil to a separate file
            let numbered_output_path = output_path.replace(".geo", &format!("_c{}.geo", coil_n));
            println!("Saving coil {} to {}...", coil_n, numbered_output_path);
            match self.save_geo(&vec![single_loop.clone()], &numbered_output_path) {
                Ok(_) => (),
                Err(error) => {
                    return Err(crate::io::IoError{file: output_path.to_string(), cause: error}.into());
                },
            };
            let txt_output_path = output_path.replace(".geo", &format!("_c{}.txt", coil_n));
            match self.save_marie_txt(&vec![single_loop.clone()], &txt_output_path) {
                Ok(_) => (),
                Err(error) => {
                    return Err(crate::io::IoError{file: output_path.to_string(), cause: error}.into());
                },
            };

            // Add the coil to the full set
            full_loops.push(single_loop);
        }

        // Save a full set of coils (often just for visualization)
        println!("Saving full array to {}", output_path);
        match self.save_geo(&full_loops, &output_path) {
            Ok(_) => (),
            Err(error) => {
                return Err(crate::io::IoError{file: output_path.to_string(), cause: error}.into());
            },
        };
        let txt_output_path = output_path.replace(".geo", ".txt");
        match self.save_marie_txt(&full_loops, &txt_output_path) {
            Ok(_) => (),
            Err(error) => {
                return Err(crate::io::IoError{file: output_path.to_string(), cause: error}.into());
            },
        };

        Ok(())
    }
}

const COL_POS : [usize; 11] = [
    4,
    14,
    32,
    54,
    66,
    70,
    72,
    74,
    86,
    98,
    110,
];

impl Method {
    /// Save a GMSH .geo file
    fn save_geo(&self, loop_vec: &Vec<Loop>, output_path: &str) -> std::io::Result<()> {
        let file = OpenOptions::new().write(true).create(true).open(&output_path)?;

        let mut file = LineWriter::new(file);


        // Write the lc
        writeln!(file, "lc = {};", self.method_args.lc)?;
        writeln!(file)?;


        // Initialize the point offsets
        let mut point_offsets = vec![0 as usize; loop_vec.len()];
        point_offsets[0] = 1;

        // Write the points
        writeln!(file, "// Points")?;
        writeln!(file, "// ------------------------------------------")?;
        for (loop_n, single_loop) in loop_vec.iter().enumerate() {
            writeln!(file, "// Coil {}", loop_n)?;
            for (point_id, point) in single_loop.points.iter().enumerate() {
                writeln!(file, "Point({}) = {{{}, {}, {}, lc}};", point_id + point_offsets[loop_n], point.x * 1e-3, point.y * 1e-3, point.z * 1e-3)?;
            }
            if loop_n < loop_vec.len() - 1 {
                point_offsets[loop_n + 1] = point_offsets[loop_n] + single_loop.points.len();
                writeln!(file)?; 
            }
        }
        writeln!(file, "// ------------------------------------------")?;
        writeln!(file)?;


        // Initialize the arc and spline offsets
        let mut arc_offsets = vec![0 as usize; loop_vec.len()];
        arc_offsets[0] = 1;
        let mut spline_offsets = vec![0 as usize; loop_vec.len()];
        spline_offsets[0] = 1;

        // Write the arcs and splines
        writeln!(file, "// Arcs and Splines")?;
        writeln!(file, "// ------------------------------------------")?;
        for (loop_n, single_loop) in loop_vec.iter().enumerate() {
            writeln!(file, "// Coil {}", loop_n)?;

            // Write the arcs
            for (arc_id, arc) in single_loop.arcs.iter().enumerate() {
                writeln!(file, "Circle({}) = {{{}, {}, {}}};", arc_id + arc_offsets[loop_n], arc.start + point_offsets[loop_n], arc.center + point_offsets[loop_n], arc.end + point_offsets[loop_n])?;
            }
            spline_offsets[loop_n] = arc_offsets[loop_n] + single_loop.arcs.len();
            writeln!(file)?;

            // Write the splines
            for (spline_id, spline) in single_loop.splines.iter().enumerate() {
                let mut spline_str = format!("Spline({}) = {{", spline_id + spline_offsets[loop_n]);
                for (point_n, point_id) in spline.points.iter().enumerate() {
                    spline_str.push_str(&(point_id + point_offsets[loop_n]).to_string());
                    if point_n < spline.points.len() - 1 {
                        spline_str.push_str(", ");
                    }
                }
                spline_str.push_str("};");
                writeln!(file, "{}", spline_str)?;
            }
            if loop_n < loop_vec.len() - 1 {
                arc_offsets[loop_n + 1] = spline_offsets[loop_n] + single_loop.splines.len();
                writeln!(file)?;
            }
        }
        writeln!(file, "// ------------------------------------------")?;
        writeln!(file)?;

        // Initialize the line loop offsets
        let mut line_loop_offsets = vec![0 as usize; loop_vec.len()];
        line_loop_offsets[0] = 1;

        // Write the line loops
        writeln!(file, "// Line Loops")?;
        writeln!(file, "// ------------------------------------------")?;
        for (loop_n, single_loop) in loop_vec.iter().enumerate() {
            writeln!(file, "// Coil {}", loop_n)?;
            let break_count = single_loop.arcs.len() / 4;
            for segment_n in 0..break_count {
                for i in 0..4 {
                    let first_arc_id = segment_n * 4 + i + arc_offsets[loop_n];
                    let second_arc_id = ((segment_n + 1) % break_count) * 4 + i + arc_offsets[loop_n];
                    
                    let first_spline_id = segment_n * 4 + i + spline_offsets[loop_n];
                    let second_spline_id = segment_n * 4 + (i + 1) % 4 + spline_offsets[loop_n];

                    let loop_id = segment_n * 4 + i + line_loop_offsets[loop_n];
                    
                    writeln!(file, "Line Loop({}) = {{-{}, {}, {}, -{}}};", 
                        loop_id, first_arc_id, first_spline_id, second_arc_id, second_spline_id)?;
                }
            }
            if loop_n < loop_vec.len() - 1 {
                line_loop_offsets[loop_n + 1] = line_loop_offsets[loop_n] + break_count * 4;
                writeln!(file)?;
            }
        }
        writeln!(file, "// ------------------------------------------")?;
        writeln!(file)?;

        // Write the ruled surfaces
        writeln!(file, "// Ruled Surfaces")?;
        writeln!(file, "// ------------------------------------------")?;
        for (loop_n, single_loop) in loop_vec.iter().enumerate() {
            writeln!(file, "// Coil {}", loop_n)?;
            let break_count = single_loop.arcs.len() / 4;
            for segment_n in 0..break_count {
                for i in 0..4 {
                    let surface_id = segment_n * 4 + i + line_loop_offsets[loop_n];
                    writeln!(file, "Ruled Surface({}) = {{{}}};", surface_id, surface_id)?;
                }
            }
            if loop_n < loop_vec.len() - 1 {
                writeln!(file)?;
            }
        }
        writeln!(file, "// ------------------------------------------")?;
        writeln!(file)?;

        
        // Write the physical lines for the ports first (first break in each loop, made of four arcs)...
        writeln!(file, "// Ports")?;
        writeln!(file, "// ------------------------------------------")?;
        for (loop_n, _) in loop_vec.iter().enumerate() {
            writeln!(file, "// Coil {}", loop_n)?;
            let arc_ids = (0..4).map(|i| i + arc_offsets[loop_n]).collect::<Vec<usize>>();
            writeln!(file, "Physical Line({}) = {{{}, {}, {}, {}}};", 
                loop_n + 1, arc_ids[0], arc_ids[1], arc_ids[2], arc_ids[3])?;
            if loop_n < loop_vec.len() - 1 {
                writeln!(file)?;
            }
        }
        writeln!(file, "// ------------------------------------------")?;
        writeln!(file)?;
            
        // ... then initialize the physical line offsets...
        let mut physical_line_offsets = vec![0 as usize; loop_vec.len()];
        physical_line_offsets[0] = loop_vec.len() + 1;


        // ... then write the lumped element physical lines (other breaks in each loop, made of four arcs)
        writeln!(file, "// Lumped Elements")?;
        writeln!(file, "// ------------------------------------------")?;
        for (loop_n, single_loop) in loop_vec.iter().enumerate() {
            writeln!(file, "// Coil {}", loop_n)?;
            let break_count = single_loop.arcs.len() / 4;
            for segment_n in 1..break_count {
                let arc_ids = (0..4).map(|i| i + segment_n * 4 + arc_offsets[loop_n]).collect::<Vec<usize>>();
                let line_id = (segment_n - 1) + physical_line_offsets[loop_n];
                writeln!(file, "Physical Line({}) = {{{}, {}, {}, {}}};", 
                    line_id, arc_ids[0], arc_ids[1], arc_ids[2], arc_ids[3])?;
            }
            if loop_n < loop_vec.len() - 1 {
                physical_line_offsets[loop_n + 1] = physical_line_offsets[loop_n] + (break_count - 1);
                writeln!(file)?;
            }
        }
        writeln!(file, "// ------------------------------------------")?;
        writeln!(file)?;

        // Write the physical surfaces (one coil)
        writeln!(file, "// Physical Surfaces")?;
        writeln!(file, "// ------------------------------------------")?;
        for (loop_n, single_loop) in loop_vec.iter().enumerate() {
            writeln!(file, "// Coil {}", loop_n)?;
            let break_count = single_loop.arcs.len() / 4;
            let mut physical_surface_str = format!("Physical Surface({}) = {{", loop_n + 1);
            for segment_n in 0..break_count {
                for i in 0..4 {
                    let surface_id = segment_n * 4 + i + line_loop_offsets[loop_n];
                    physical_surface_str.push_str(&surface_id.to_string());
                    if segment_n < break_count - 1 || i < 3 {
                        physical_surface_str.push_str(", ");
                    }
                }
            }
            physical_surface_str.push_str("};");
            writeln!(file, "{}", physical_surface_str)?;
            if loop_n < loop_vec.len() - 1 {
                writeln!(file)?;
            }
        }
        writeln!(file, "// ------------------------------------------")?;
        writeln!(file)?;

        writeln!(file, "Coherence Mesh;")?;

        Ok(())
    }

    /// Save a MARIE .txt file for ports and lumped elements
    fn save_marie_txt(&self, loop_vec: &Vec<Loop>, output_path: &str) -> std::io::Result<()> {
        let file = OpenOptions::new().write(true).create(true).open(&output_path)?;

        let mut file = LineWriter::new(file);

        // Write the ports
        for (loop_n, _) in loop_vec.iter().enumerate() {
            let mut line_str = format!("{}", loop_n + 1);
            line_str.push_str(&" ".repeat(COL_POS[0] - line_str.len()));

            line_str.push_str(&"port");
            line_str.push_str(&" ".repeat(COL_POS[1] - line_str.len()));

            // TODO: Figure the rest out

            writeln!(file, "{}", line_str)?;
        }

        // ... then initialize the physical line offsets...
        let mut physical_line_offsets = vec![0 as usize; loop_vec.len()];
        physical_line_offsets[0] = loop_vec.len() + 1;

        // ... then write the lumped elements
        for (loop_n, single_loop) in loop_vec.iter().enumerate() {
            let break_count = single_loop.arcs.len() / 4;
            let capacitor_count = break_count - 2;
            for segment_n in 1..break_count {
                let mut line_str = format!("{}", (segment_n - 1) + physical_line_offsets[loop_n]);
                line_str.push_str(&" ".repeat(COL_POS[0] - line_str.len()));
                
                line_str.push_str(&"element");
                line_str.push_str(&" ".repeat(COL_POS[1] - line_str.len()));

                // TODO: Figure the rest out

                writeln!(file, "{}", line_str)?;
            }
            if loop_n < loop_vec.len() - 1 {
                physical_line_offsets[loop_n + 1] = physical_line_offsets[loop_n] + (break_count - 1);
            }
        }

        Ok(())
    }
        
}   
