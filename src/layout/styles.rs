/*!
 * This is the layout styles module.
 * Adding new styles should be done here.
 * 
 * New styles need to be added to the CLI and to layout system (`do_layout` etc.) handling.
 * 
 * # Adding to CLI:
 * - Add a new enum variant to `LayoutStyleCliEnum`
 * - Add the constructor handling in the match case of the `construct_layout_style` 
 * 
 * # Adding to layout system:
 * - Add a new enum variant to `LayoutStyle`
 * - Implement the trait `IsStyle` for the new style
 * 
 */

mod iterative_circle;

use enum_dispatch::enum_dispatch;
use clap::ValueEnum;

use crate::layout;

/// Layout styles CLI enum.
/// Add a new style here to add it to the CLI.
#[derive(Debug, Clone, ValueEnum)]
#[clap(rename_all = "kebab_case")]
pub enum LayoutStyleCliEnum {
    /// Basic circular layout, based on Monika Sliwak's MATLAB prototype.
    IterativeCircle,
}

impl LayoutStyleCliEnum {
    /// Construct a layout style from the CLI enum.
    /// Takes a `LayoutStyleCliEnum` and returns a `Result` with the `LayoutStyle` or an `Err`.
    pub fn construct(&self, layout_args: layout::LayoutArgs) -> Result<LayoutStyle, String> {
        match self {
            // Match the CLI enum variant and construct the corresponding layout style.
            LayoutStyleCliEnum::IterativeCircle => {
                let style = iterative_circle::Style::new(layout_args)?;
                Ok(LayoutStyle::IterativeCircle(style))
            },
        }
    }
}

/// Layout styles enum.
/// To add a new style:
/// implement the `IsStyle` trait for it,
/// include it here and the `LayoutStyleCliEnum` enum,
/// and add handling for its constructor.
#[derive(Debug)]
#[enum_dispatch(IsStyle)]
pub enum LayoutStyle {
    /// Basic circular layout, based on Monika Sliwak's MATLAB prototype.
    IterativeCircle(iterative_circle::Style),
}

/// Layout style trait.
/// This trait defines the functions that all layout styles must implement.
/// To add a new style:
/// implement this trait for it,
/// include it in the `LayoutStyle` and `LayoutStyleCliEnum` enums,
/// and add handling for its constructor.
#[enum_dispatch]
pub trait IsStyle {
    /// Run the layout process with the given arguments.
    /// Uses the `layout` module.
    /// Takes parsed arguments (from `parse_layout_args` or future GUI).
    /// Returns a `Result` with the `layout::Layout` or an `Err`.
    fn do_layout(&self) -> Result<layout::Layout, String>;

    /// Get the name of the layout style.
    fn get_style_name(&self) -> String;
}
