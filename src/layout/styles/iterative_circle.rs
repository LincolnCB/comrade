use crate::layout;
use crate::layout::styles;

/// Iterative Circle Style struct.
/// This struct contains all the parameters for the Iterative Circle layout style.
pub struct Style {
    /// Arguments for the layout process.
    layout_args: layout::LayoutArgs,
}


impl Style {
    /// Create a new Iterative Circle layout style.
    /// Takes the `layout::LayoutArgs` and returns a `Result` with the `Style` or an `Error`.
    pub fn new(layout_args: layout::LayoutArgs) -> Result<Style, String> {
        Ok(Style{layout_args})
    }
}

impl styles::LayoutStyle for Style {
    /// Run the layout process with the given arguments.
    /// Uses the `layout` module.
    /// Takes parsed arguments (from `parse_layout_args` or future GUI).
    /// Returns a `Result` with the `layout::Layout` or an `Error`.
    fn do_layout(&self) -> Result<layout::Layout, String> {
        println!("Dummy DO ITERATIVE CIRCLE LAYOUT");
        Ok(layout::Layout{})
    }

    /// Get the name of the layout style.
    fn get_style_name(&self) -> String {
        "Iterative Circle".to_string()
    }
}
