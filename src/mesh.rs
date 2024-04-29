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
pub use cfg::MeshTarget;
// Re-export meshing methods
pub use methods::{
    MethodEnum,
    MeshMethodTrait,
};

pub fn do_mesh(mesh_target: &MeshTarget, layout_in: &layout::Layout) -> ProcResult<()> {
    // Extract the mesh method from target
    let mesh_method = &mesh_target.method;

    println!("Meshing...");

    // Run the meshing method
    println!("Running meshing method: {}", mesh_method.get_method_display_name());
    mesh_method.save_mesh(&layout_in, &mesh_target.output_path)
}
