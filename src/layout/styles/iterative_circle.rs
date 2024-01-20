use crate::layout;
use crate::layout::styles;

/// Iterative Circle Style struct.
/// This struct contains all the parameters for the Iterative Circle layout style.
#[derive(Debug)]
#[allow(dead_code)]
pub struct Style {
    /// Arguments for the layout process.
    layout_args: layout::LayoutArgs,
}


impl Style {
    /// Create a new Iterative Circle layout style.
    /// Takes the `layout::LayoutArgs` and returns a `Result` with the `Style` or an `Err`.
    pub fn new(layout_args: layout::LayoutArgs) -> crate::Result<Style> {
        Ok(Style{layout_args})
    }
}

impl styles::IsStyle for Style {
    /// Get the name of the layout style.
    fn get_style_name(&self) -> String {
        "Iterative Circle".to_string()
    }

    /// Run the layout process with the given arguments.
    /// Uses the `layout` module.
    /// Takes parsed arguments (from `parse_layout_args` or future GUI).
    /// Returns a `Result` with the `layout::Layout` or an `Err`.
    #[allow(unused_variables)]
    #[allow(unused_mut)]
    #[allow(unused_assignments)]
    #[allow(dead_code)]
    fn do_layout(&self, surface: &layout::Surface) -> crate::Result<layout::Layout> {
        println!("Dummy DO ITERATIVE CIRCLE LAYOUT");
        let mut layout_out = layout::Layout::new();

        // Temporary coil size estimate
        let coil_area = surface.area / self.layout_args.coil_count as f32;
        let coil_radius = (coil_area / std::f32::consts::PI).sqrt();

        // TEMP CENTER POINT: -1.305, 1.6107, 29.919

        for _ in 0..self.layout_args.coil_count {
            let mut points = Vec::new();
            let coil = layout::Coil::new_from_vec(points)?;
            layout_out.coils.push(coil);
        }

        Ok(layout_out)
    }
}
