mod proc_errors;
mod cfg;
mod methods;

use crate::layout;

// Re-export errors
pub use proc_errors::{
    MeshError,
    ProcResult,
    err_str,
};
// Re-export cfg handling
pub use cfg::{
    MeshArgs,
    MeshTarget,
};
// Re-export meshing methods
pub use methods::{
    MeshChoice,
    MeshMethod,
};

pub fn do_mesh(mesh_target: &MeshTarget, layout_in: &layout::Layout) -> ProcResult<()> {

    // Extract the mesh method and arguments from target
    let mesh_method = &mesh_target.mesh_method;
    let mesh_args = &mesh_target.mesh_args;

    println!("Meshing...");

    // Run the meshing method
    println!("Running meshing method: {}", mesh_method.get_method_name());
    mesh_method.save_mesh(&layout_in, &mesh_args.output_path)
}
