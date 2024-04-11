use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use serde::de::{Error, Unexpected, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;

#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Hash)]
pub struct Colour(pub vello::peniko::Color);

impl ToTokens for Colour {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let r = self.0.r;
        let g = self.0.g;
        let b = self.0.b;
        let a = self.0.a;
        tokens.extend(quote!(::gui::gui_core::Colour::rgba8(#r, #g, #b, #a)))
    }
}

impl Colour {
    pub const fn rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Colour(vello::peniko::Color { r, g, b, a })
    }
}

struct ColourVisitor;

impl<'de> Visitor<'de> for ColourVisitor {
    type Value = Colour;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("a colour as either in hexidecimal CSS style of the form #RGB, #RGBA, #RRGGBB, #RRGGBBAA or the name of an SVG color such as \"aliceblue\"")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        vello::peniko::Color::parse(v)
            .ok_or_else(|| Error::invalid_value(Unexpected::Other("unrecognized colour."), &self))
            .map(Colour)
    }
}

impl<'de> Deserialize<'de> for Colour {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ColourVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::Colour;
    use quote::ToTokens;
    use vello::peniko::Color;

    #[test]
    pub fn colour_deserialize_known_name() {
        let colour: Colour = serde_yaml::from_str(r##""blue""##).unwrap();
        assert_eq!(colour.0, Color::BLUE);
    }

    #[test]
    pub fn colour_deserialize_hex_rgb() {
        let colour: Colour = serde_yaml::from_str(r##""#FF0000""##).unwrap();
        assert_eq!(colour.0, Color::RED);
    }

    #[test]
    pub fn colour_deserialize_hex_rgba() {
        let colour: Colour = serde_yaml::from_str(r##""#FF000080""##).unwrap();
        assert_eq!(
            colour.0,
            Color {
                r: 255,
                g: 0,
                b: 0,
                a: 128,
            }
        );
    }

    #[test]
    pub fn colour_deserialize_invalid() {
        let result: Result<Colour, _> = serde_yaml::from_str(r##""invalid""##);
        assert!(result.is_err());
    }

    #[test]
    pub fn colour_to_tokens() {
        let colour = Colour(Color::GREEN);
        let mut tokens = proc_macro2::TokenStream::new();
        colour.to_tokens(&mut tokens);
        assert_eq!(
            tokens.to_string(),
            ":: gui :: gui_core :: Colour :: rgba8 (0u8 , 128u8 , 0u8 , 255u8)"
        );
    }
}
