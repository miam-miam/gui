pub mod button;
mod hvstack;
mod text;

pub use button::Button;

pub use text::Text;

pub use hvstack::HVStack;

#[cfg(test)]
mod tests {
    #[test]
    fn parse_simple() {
        let yaml = include_str!("simple.yaml");
        let _ser: gui_core::parse::GUIDeclaration = serde_yaml::from_str(yaml).unwrap();
    }
}
