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
    #[serde(default = "MethodCfg::default_single_surface")]
    single_surface: bool,
    #[serde(default = "MethodCfg::default_polygonal")]
    polygonal: bool,
    #[serde(default = "MethodCfg::default_poly_count", alias = "spline_count")]
    poly_count: usize,
    #[serde(default = "MethodCfg::default_lc")]
    lc: f32,
    #[serde(default = "MethodCfg::default_larmor_mhz")]
    larmor_mhz: f32,
    #[serde(default = "GeoVector::zero")]
    origin_offset: GeoVector,
}
impl MethodCfg {
    pub fn default_break_count() -> usize {
        4
    }
    pub fn default_angle_shift() -> f32 {
        0.0
    }
    pub fn default_single_surface() -> bool {
        true
    }
    pub fn default_polygonal() -> bool {
        false
    }
    pub fn default_poly_count() -> usize {
        4
    }
    pub fn default_lc() -> f32 {
        0.002
    }
    pub fn default_larmor_mhz() -> f32 {
        127.73
    }
    pub fn default() -> Self {
        MethodCfg{
            break_count: Self::default_break_count(),
            angle_shift: Self::default_angle_shift(),
            single_surface: Self::default_single_surface(),
            polygonal: Self::default_polygonal(),
            poly_count: Self::default_poly_count(),
            lc: Self::default_lc(),
            larmor_mhz: Self::default_larmor_mhz(),
            origin_offset: GeoVector::zero(),
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
    self_inductance_nh: f32,
}
impl Loop {
    pub fn new() -> Self {
        Loop{points: Vec::new(), arcs: Vec::new(), splines: Vec::new(), self_inductance_nh: 0.0}
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

        let poly_count = self.method_args.poly_count;

        let mut full_loops = Vec::<Loop>::new();
        
        // Mesh each coil
        for (coil_n, coil) in layout.coils.iter().enumerate() {
            println!("Coil {}...", coil_n);

            let radius = coil.wire_radius;

            // Initialize the GMSH vectors
            let mut single_loop = Loop::new();
            single_loop.self_inductance_nh = coil.self_inductance(1.0);
            
            // Add the radial polygon points for each coil vertex (and center, used for arcs)
            let center = coil.center;
            for vertex in coil.vertices.iter() {
                let point = &vertex.point;

                // Get the relevant vectors
                let vec_to_point = *point - center;
                let up_vec = vertex.wire_radius_normal;
                let out_vec = vec_to_point.rej_onto(&up_vec).normalize();
                
                // Add the spline points to the list
                for i in 0..poly_count {
                    let theta = (i as f32 - 0.5) * 2.0 * PI / poly_count as f32; // -0.5 gives a flat bottom
                    let point = vertex.point + up_vec * radius * theta.cos() + out_vec * radius * theta.sin();
                    single_loop.points.push(point + self.method_args.origin_offset);
                }
                // Add the wire point to the list (some may be unused)
                single_loop.points.push(vertex.point + self.method_args.origin_offset);
            }

            // Add two capacitor breaks on either side of the first binned break (the port)
            // Position them at the first points in either direction from the port that are at least 2*lc away
            let port_id = if let Some(id) = coil.port {
                id
            } else {
                0
            };

            // Upper side capacitor break:
            let mut upper_capacitor_break_id = port_id;
            let mut distance = 0.0;
            while distance < 2.0 * self.method_args.lc {
                let previously_checked_id = upper_capacitor_break_id;
                upper_capacitor_break_id = coil.vertices[upper_capacitor_break_id].next_id;
                distance += (coil.vertices[upper_capacitor_break_id].point - coil.vertices[previously_checked_id].point).norm();
                if coil.breaks.len() > 1 && upper_capacitor_break_id == coil.breaks[0] {
                    mesh::err_str("Math error: Nearby capacitor break (positive idx direction) not found before first break -- lc too large")?;
                }
            }
            
            // Lower side capacitor break:
            let mut lower_capacitor_break_id = port_id;
            let mut distance = 0.0;
            while distance < 2.0 * self.method_args.lc {
                let previously_checked_id = lower_capacitor_break_id;
                lower_capacitor_break_id = coil.vertices[lower_capacitor_break_id].prev_id;
                distance += (coil.vertices[lower_capacitor_break_id].point - coil.vertices[previously_checked_id].point).norm();
                if coil.breaks.len() > 1 && lower_capacitor_break_id == coil.breaks[coil.breaks.len() - 1] {
                    mesh::err_str("Math error: Nearby capacitor break (negative idx direction) not found before last break -- lc too large")?;
                }
            }

            let mut break_points = vec![port_id, upper_capacitor_break_id];
            break_points.extend(coil.breaks.clone());
            break_points.push(lower_capacitor_break_id);

            // Add the arcs
            for id in break_points.iter() {
                // Add the arcs per point (equal to spline poly count, 4 by default)
                for i in 0..poly_count {
                    let start = id * (poly_count + 1) + i;
                    let center = id * (poly_count + 1) + poly_count;
                    let end = id * (poly_count + 1) + (i + 1) % poly_count;
                    single_loop.arcs.push(Arc{start, center, end});
                }
            }

            // Add the splines.
            for break_number in 0..break_points.len() {
                
                let id = break_points[break_number];
                let next_id = break_points[(break_number + 1) % break_points.len()];
                // Add the splines per layout point
                for i in 0..poly_count {
                    let mut spline_points = Vec::<usize>::new();

                    // Handle potential wraparound
                    if next_id < id {
                        for j in id..coil.vertices.len() {
                            spline_points.push(j * (poly_count + 1) + i);
                        }
                        for j in 0..=next_id {
                            spline_points.push(j * (poly_count + 1) + i);
                        }
                    } else {
                        for j in id..=next_id {
                            spline_points.push(j * (poly_count + 1) + i);
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

const COL_WIDTH : [usize; 11] = [
    4,
    8,
    10,
    12,
    3,
    3,
    2,
    2,
    6,
    6,
    8,
];

impl Method {
    /// Save a GMSH .geo file
    fn save_geo(&self, loop_vec: &Vec<Loop>, output_path: &str) -> std::io::Result<()> {
        let file = OpenOptions::new().write(true).create(true).truncate(true).open(&output_path)?;

        let mut file = LineWriter::new(file);

        let poly_count = self.method_args.poly_count;

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
                if self.method_args.polygonal {
                    writeln!(file, "Line({}) = {{{}, {}}};", arc_id + arc_offsets[loop_n], arc.start + point_offsets[loop_n], arc.end + point_offsets[loop_n])?;
                } else {
                    writeln!(file, "Circle({}) = {{{}, {}, {}}};", arc_id + arc_offsets[loop_n], arc.start + point_offsets[loop_n], arc.center + point_offsets[loop_n], arc.end + point_offsets[loop_n])?;
                }
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
            let break_count = single_loop.arcs.len() / poly_count;
            for segment_n in 0..break_count {
                for i in 0..poly_count {
                    let first_arc_id = segment_n * poly_count + i + arc_offsets[loop_n];
                    let second_arc_id = ((segment_n + 1) % break_count) * poly_count + i + arc_offsets[loop_n];
                    
                    let first_spline_id = segment_n * poly_count + i + spline_offsets[loop_n];
                    let second_spline_id = segment_n * poly_count + (i + 1) % poly_count + spline_offsets[loop_n];

                    let loop_id = segment_n * poly_count + i + line_loop_offsets[loop_n];
                    
                    writeln!(file, "Line Loop({}) = {{-{}, {}, {}, -{}}};", 
                        loop_id, first_arc_id, first_spline_id, second_arc_id, second_spline_id)?;
                }
            }
            if loop_n < loop_vec.len() - 1 {
                line_loop_offsets[loop_n + 1] = line_loop_offsets[loop_n] + break_count * poly_count;
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
            let break_count = single_loop.arcs.len() / poly_count;
            for segment_n in 0..break_count {
                for i in 0..poly_count {
                    let surface_id = segment_n * poly_count + i + line_loop_offsets[loop_n];
                    writeln!(file, "Ruled Surface({}) = {{{}}};", surface_id, surface_id)?;
                }
            }
            if loop_n < loop_vec.len() - 1 {
                writeln!(file)?;
            }
        }
        writeln!(file, "// ------------------------------------------")?;
        writeln!(file)?;

        
        // Write the physical lines for the ports first (first break in each loop, made of arcs)...
        writeln!(file, "// Ports")?;
        writeln!(file, "// ------------------------------------------")?;
        for (loop_n, _) in loop_vec.iter().enumerate() {
            writeln!(file, "// Coil {}", loop_n)?;
            let arc_ids = (0..poly_count).map(|i| i + arc_offsets[loop_n]).collect::<Vec<usize>>();
            let mut physical_line_str = format!("Physical Line({}) = {{", loop_n + 1);
            for (i, arc_id) in arc_ids.iter().enumerate() {
                physical_line_str.push_str(&arc_id.to_string());
                if i < arc_ids.len() - 1 {
                    physical_line_str.push_str(", ");
                }
            }
            physical_line_str.push_str("};");
            writeln!(file, "{}", physical_line_str)?;
            if loop_n < loop_vec.len() - 1 {
                writeln!(file)?;
            }
        }
        writeln!(file, "// ------------------------------------------")?;
        writeln!(file)?;
            
        // ... then initialize the physical line offsets...
        let mut physical_line_offsets = vec![0 as usize; loop_vec.len()];
        physical_line_offsets[0] = loop_vec.len() + 1;


        // ... then write the lumped element physical lines (other breaks in each loop, made of arcs)
        writeln!(file, "// Lumped Elements")?;
        writeln!(file, "// ------------------------------------------")?;
        for (loop_n, single_loop) in loop_vec.iter().enumerate() {
            writeln!(file, "// Coil {}", loop_n)?;
            let break_count = single_loop.arcs.len() / poly_count;
            for segment_n in 1..break_count {
                let arc_ids = (0..poly_count).map(|i| i + segment_n * poly_count + arc_offsets[loop_n]).collect::<Vec<usize>>();
                let line_id = (segment_n - 1) + physical_line_offsets[loop_n];
                let mut physical_line_str = format!("Physical Line({}) = {{", line_id);
                for (i, arc_id) in arc_ids.iter().enumerate() {
                    physical_line_str.push_str(&arc_id.to_string());
                    if i < arc_ids.len() - 1 {
                        physical_line_str.push_str(", ");
                    }
                }
                physical_line_str.push_str("};");
                writeln!(file, "{}", physical_line_str)?;
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
        let single_surface = self.method_args.single_surface;
        let mut physical_surface_str = "".to_string();
        for (loop_n, single_loop) in loop_vec.iter().enumerate() {
            let break_count = single_loop.arcs.len() / poly_count;
            if !single_surface || loop_n == 0 {
                if single_surface { writeln!(file, "// Single Surface")?; } else { writeln!(file, "// Coil {}", loop_n)?; }
                physical_surface_str = format!("Physical Surface({}) = {{", loop_n + 1);
            }
            for segment_n in 0..break_count {
                for i in 0..poly_count {
                    let surface_id = segment_n * poly_count + i + line_loop_offsets[loop_n];
                    physical_surface_str.push_str(&surface_id.to_string());
                    if segment_n < break_count - 1 || i < (poly_count - 1) {
                        physical_surface_str.push_str(", ");
                    }
                }
            }
            if !single_surface || loop_n == loop_vec.len() - 1 {
                physical_surface_str.push_str("};");
                writeln!(file, "{}", physical_surface_str)?;
                if loop_n < loop_vec.len() - 1 {
                    writeln!(file)?;
                }
            } else {
                physical_surface_str.push_str(", ");
            }
        }
        writeln!(file, "// ------------------------------------------")?;
        writeln!(file)?;

        writeln!(file, "Coherence Mesh;")?;

        Ok(())
    }

    /// Save a MARIE .txt file for ports and lumped elements
    fn save_marie_txt(&self, loop_vec: &Vec<Loop>, output_path: &str) -> std::io::Result<()> {
        let file = OpenOptions::new().write(true).create(true).truncate(true).open(&output_path)?;
        let push_column = |line_str: &mut String, input: &str, col_width: usize| {
            line_str.push_str(input);
            if input.len() < col_width {
                line_str.push_str(&" ".repeat(col_width - input.len()));
            };
        };

        let mut file = LineWriter::new(file);

        let poly_count = self.method_args.poly_count;

        // Write the ports
        for (loop_n, _) in loop_vec.iter().enumerate() {
            let mut line_str = "".to_string();
            push_column(&mut line_str, &format!("{}", loop_n + 1), COL_WIDTH[0]);
            push_column(&mut line_str, "port", COL_WIDTH[1]);
            push_column(&mut line_str, "resistor", COL_WIDTH[2]);
            push_column(&mut line_str, "0", COL_WIDTH[3]);
            push_column(&mut line_str, "[]", COL_WIDTH[4]);
            push_column(&mut line_str, "[]", COL_WIDTH[5]);
            push_column(&mut line_str, "0", COL_WIDTH[6]);
            push_column(&mut line_str, "0", COL_WIDTH[7]);
            push_column(&mut line_str, "1e-12", COL_WIDTH[8]);
            push_column(&mut line_str, "1e-12", COL_WIDTH[9]);
            push_column(&mut line_str, "150e-12", COL_WIDTH[10]);
            line_str.push_str(&format!("{}", loop_n + 1));

            writeln!(file, "{}", line_str)?;
        }

        // ... then initialize the physical line offsets...
        let mut physical_line_offsets = vec![0 as usize; loop_vec.len()];
        physical_line_offsets[0] = loop_vec.len() + 1;

        // ... then write the lumped elements
        for (loop_n, single_loop) in loop_vec.iter().enumerate() {
            let break_count = single_loop.arcs.len() / poly_count;
            let capacitor_count = break_count - 2;
            let break_cap_pf = capacitor_count as f32 * 1.0e9 / ((2.0 * PI * self.method_args.larmor_mhz).powi(2) * single_loop.self_inductance_nh);
            for segment_n in 1..break_count {

                let mut line_str = "".to_string();
                push_column(&mut line_str, &format!("{}", segment_n - 1 + physical_line_offsets[loop_n]), COL_WIDTH[0]);
                push_column(&mut line_str, "element", COL_WIDTH[1]);
                push_column(&mut line_str, "capacitor", COL_WIDTH[2]);
                if segment_n == 1 || segment_n == break_count - 1 {
                    push_column(&mut line_str, &format!("{:.2}e-12", (2.0 * break_cap_pf)), COL_WIDTH[3]);
                } else {
                    push_column(&mut line_str, &format!("{:.2}e-12", break_cap_pf), COL_WIDTH[3]);
                }
                push_column(&mut line_str, "[]", COL_WIDTH[4]);
                push_column(&mut line_str, "[]", COL_WIDTH[5]);
                push_column(&mut line_str, "0", COL_WIDTH[6]);
                push_column(&mut line_str, "0", COL_WIDTH[7]);
                push_column(&mut line_str, "1e-12", COL_WIDTH[8]);
                push_column(&mut line_str, "1e-12", COL_WIDTH[9]);
                push_column(&mut line_str, "150e-12", COL_WIDTH[10]);
                if segment_n == 1 || segment_n == break_count - 1 {
                    line_str.push_str(&format!("{}", loop_vec.len() + 2 * loop_n + 1));
                } else {
                    line_str.push_str(&format!("{}", loop_vec.len() + 2 * loop_n + 2));
                }

                writeln!(file, "{}", line_str)?;
            }
            if loop_n < loop_vec.len() - 1 {
                physical_line_offsets[loop_n + 1] = physical_line_offsets[loop_n] + (break_count - 1);
            }
        }

        Ok(())
    }
        
}   
