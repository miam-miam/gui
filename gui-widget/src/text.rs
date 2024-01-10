use gui_core::widget::WidgetBuilder;
use gui_core::{Colour, Var};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TextBuilder {
    pub text: Option<Var<String>>,
    pub colour: Option<Var<Colour>>,
    pub font: Option<Var<String>>,
    pub size: Option<Var<u32>>,
}

#[typetag::deserialize(name = "Text")]
impl WidgetBuilder for TextBuilder {}
