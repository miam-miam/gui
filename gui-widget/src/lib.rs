pub mod button;
mod hbox;
mod text;

pub use button::Button;

pub use text::Text;

#[cfg(test)]
mod tests {
    #[test]
    fn parse_simple() {
        let yaml = include_str!("simple.yaml");
        let _ser: gui_core::parse::GUIDeclaration = serde_yaml::from_str(yaml).unwrap();
    }
}
