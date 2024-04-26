/*!
 * This is the meshing methods module.
 * Adding new methods should be done here.
 * 
 * New methods need:
 * - A struct implementing `MeshMethodTrait`
 * - An enum variant containing that struct in `MethodEnum`
 * - A constructor arg_name and function in `MESH_TARGET_CONSTRUCTION`
 * 
 */

use enum_dispatch::enum_dispatch;
use serde::{Serialize, Deserialize};

use crate::{
    layout,
    mesh,
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
/// and implement the `MeshMethodTrait` trait for it.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[enum_dispatch(MeshMethodTrait)]
#[serde(tag = "name", content = "args")]
pub enum MethodEnum {
    /// Meshing method based on STL polygons.
    #[serde(rename = "stl_polygons")]
    StlPolygons(stl_polygons::Method),
    /// Meshing method that creates a slot for CAD models.
    #[serde(rename = "stl_slot")]
    StlSlot(stl_slot::Method),
    /// Meshing method that creates a mesh for Marie's GMesh.
    #[serde(rename = "gmsh")]
    Gmsh(gmsh::Method),
}

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
/// include it in the `MethodEnum` enum,
/// add handling for its constructor in `MESH_TARGET_CONSTRUCTION`,
/// and implement this trait for it.
#[enum_dispatch] // This is a macro that allows the enum to be used in a trait object-like way
pub trait MeshMethodTrait {
    /// Get the name of the meshing method.
    fn get_method_name(&self) -> &'static str;

    /// Get the output file extension for the meshing method.
    fn get_output_extension(&self) -> &'static str;

    /// Save the mesh to a file.
    fn save_mesh(&self, layout: &layout::Layout, output_path: &str) -> mesh::ProcResult<()>;
}
