// Structures are declared as modules to allow for private fields and file management.
mod point;
mod vector;
mod plane;
mod surface;

// Re-export the modules
pub use point::*;
pub use vector::*;
pub use plane::*;
pub use surface::*;

/// Angle type (alias for f32).
pub type Angle = f32;
