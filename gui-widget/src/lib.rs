pub use button::Button;
pub use comp_holder::CompHolder;
pub use hvstack::HVStack;
pub use image::ImageWidget;
pub use text::Text;

pub mod button;
mod comp_holder;
mod hvstack;
mod image;
mod text;

/// Hack to allow widget paths to be asserted as unlike any other widget library
/// this one gets imported through the gui crate
#[doc(hidden)]
mod gui_widget {
    pub use super::*;
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_simple() {
        let yaml = include_str!("simple.yaml");
        let _ser: gui_custom::parse::GUIDeclaration = serde_yaml::from_str(yaml).unwrap();
    }
}
