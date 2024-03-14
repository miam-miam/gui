pub mod button;
mod comp_holder;
mod hvstack;
mod image;
mod text;

pub use button::Button;

pub use text::Text;

pub use hvstack::HVStack;

pub use comp_holder::CompHolder;

pub use image::ImageWidget;

#[cfg(test)]
mod tests {
    #[test]
    fn parse_simple() {
        let yaml = include_str!("simple.yaml");
        let _ser: gui_core::parse::GUIDeclaration = serde_yaml::from_str(yaml).unwrap();
    }
}
