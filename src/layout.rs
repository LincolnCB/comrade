use clap::Args;

/// Load a STL file from the inut path.
/// Uses the `stl_io` crate.
/// Returns a `Result` with the `Mesh` or an `Error`
pub fn load_stl() {
    println!("Dummy load_stl");
}

#[derive(Debug, Args)]
pub struct LayoutOnlyCli {

    #[arg(short, long = "count")]
    /// Coil count for the layout.
    pub coil_count: u64,

    #[arg(short, long = "mesh")]
    /// Output a mesh file with the same output name.
    pub mesh: bool,

    // TODO: Inductive decoupling, default 11dB

    // TODO: Output mesh type, default .ply? .stl?

}

