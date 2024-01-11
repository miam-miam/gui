use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use serde::de::{Error, Unexpected, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt::Formatter;

#[derive(Debug, Copy, Clone, Default)]
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
