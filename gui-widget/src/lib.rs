mod text;

pub use text::Text;

#[cfg(test)]
mod tests {
    #[test]
    fn parse_simple() {
        let yaml = include_str!("simple.yaml");
        let ser: gui_core::parse::GUIDeclaration =
            serde_yaml::from_str(yaml).expect("TODO: panic message");
        dbg!(ser);
    }
}
