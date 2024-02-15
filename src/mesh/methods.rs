/*!
 * This is the meshing methods module.
 * Adding new methods should be done here.
 * 
 * New methods need:
 * - A struct implementing `MeshMethod`
 * - An enum variant containing that struct in `MeshChoice`
 * - A constructor arg_name and function in `MESH_TARGET_CONSTRUCTION`
 * 
 */

use enum_dispatch::enum_dispatch;

use crate::{
    layout,
    mesh,
    args,
};

//
// ------------------------------------------------------------
// Code that requires modification to add a new meshing method
//      |
//      V
//

// Source files for the meshing methods
mod stl_polygons;
mod stl_slot;
mod gmsh;

/// Meshing methods enum.
/// To add a new method:
/// include it here,
/// add handling for its constructor in `MESH_TARGET_CONSTRUCTION`,
/// and implement the `MeshMethod` trait for it.
#[derive(Debug)]
#[enum_dispatch(MeshMethod)]
pub enum MeshChoice {
    /// Meshing method based on STL polygons.
    StlPolygons(stl_polygons::Method),
    /// Meshing method that creates a slot for CAD models.
    StlSlot(stl_slot::Method),
    /// Meshing method that creates a mesh for Marie's GMesh.
    Gmsh(gmsh::Method),
}

/// Meshing construction array -- Written out in once place for easy modification.
/// To add a new method:
/// include it in the `MeshChoice` enum,
/// add handling for its constructor here,
/// and implement the `MeshMethod` trait for it.
const MESH_TARGET_CONSTRUCTION: &[MeshConstructor] = &[
    // Example meshing constructor.
    MeshConstructor{
        arg_name: "stl_polygons", 
        constructor: || {Ok(MeshChoice::StlPolygons(stl_polygons::Method::new()?))},
    },
    // Slot meshing constructor.
    MeshConstructor{
        arg_name: "stl_slot", 
        constructor: || {Ok(MeshChoice::StlSlot(stl_slot::Method::new()?))},
    },
    // Marie's GMesh meshing constructor.
    MeshConstructor{
        arg_name: "gmsh", 
        constructor: || {Ok(MeshChoice::Gmsh(gmsh::Method::new()?))},
    },
];

//
// ------------------------------------------------------------
// Traits and structs that don't need modification,
// but are references for adding a new meshing method
//      |
//      V
//

/// Meshing method trait.
/// This trait must be implemented for all meshing methods.
/// To add a new method:
/// include it in the `MeshChoice` enum,
/// add handling for its constructor in `MESH_TARGET_CONSTRUCTION`,
/// and implement this trait for it.
#[enum_dispatch] // This is a macro that allows the enum to be used in a trait object-like way
pub trait MeshMethod {
    /// Get the name of the meshing method.
    fn get_method_name(&self) -> String;

    /// Get the output file extension for the meshing method.
    fn get_output_extension(&self) -> String;

    /// Parse the method config file
    fn parse_method_cfg(&mut self, method_cfg_file: &str) -> args::ProcResult<()>;

    /// Save the mesh to a file.
    fn save_mesh(&self, layout: &layout::Layout, output_path: &str) -> mesh::ProcResult<()>;
}

/// Meshing method constructor.
/// Used to construct a meshing method from a config file.
struct MeshConstructor {
    /// Name of the meshing method.
    arg_name: &'static str,
    /// Constructor function.
    constructor: fn() -> args::ProcResult<MeshChoice>,
}

//
// ------------------------------------------------------------
// Functions and structs with no modification or reference needed
//      |
//      V
//

/// Meshing target construction
impl MeshChoice {
    /// Construct a meshing target from a name (given in the config file).
    pub fn from_name(arg_name: &str) -> args::ProcResult<Self> {
        for constructor in MESH_TARGET_CONSTRUCTION {
            if constructor.arg_name == arg_name {
                return (constructor.constructor)();
            }
        }
        
        let mut error_str = format!("Meshing method not found: {arg_name}\n");
        error_str.push_str("Available methods:\n");
        for constructor in MESH_TARGET_CONSTRUCTION {
            error_str.push_str(&format!("    {}\n", constructor.arg_name));
        }
        args::err_str(&error_str)
    }
}
