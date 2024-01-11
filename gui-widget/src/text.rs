use gui_core::common::text;
use gui_core::common::text::ParleyBrush;
use gui_core::parley::layout::Layout;
use gui_core::parley::style::StyleProperty;
use gui_core::parley::{layout, LayoutContext};
use gui_core::vello::kurbo::Affine;
use gui_core::vello::peniko::{Brush, Color};
use gui_core::widget::{Widget, WidgetBuilder};
use gui_core::{Colour, FontContext, SceneBuilder, Var};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use serde::Deserialize;

pub struct Text {
    text: String,
    layout: Option<Layout<ParleyBrush>>,
}

impl Text {
    pub fn new(text: String) -> Self {
        Text { text, layout: None }
    }

    fn build(&mut self, fcx: &mut FontContext) {
        let mut lcx = LayoutContext::new();
        let mut layout_builder = lcx.ranged_builder(fcx, &self.text, 1.0);
        layout_builder.push_default(&StyleProperty::FontSize(12.0));
        layout_builder.push_default(&StyleProperty::Brush(ParleyBrush(Brush::Solid(
            Color::rgb8(255, 255, 255),
        ))));
        self.layout = Some(layout_builder.build());
    }
}

impl Widget for Text {
    fn render(&mut self, mut scene: SceneBuilder, fcx: &mut FontContext) {
        if self.layout.is_none() {
            self.build(fcx);
        }

        let layout = self.layout.as_mut().unwrap();
        layout.break_all_lines(None, layout::Alignment::Start);
        text::render_text(&mut scene, Affine::translate((0.0, 0.0)), layout);
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct TextBuilder {
    pub text: Option<Var<String>>,
    pub colour: Option<Var<Colour>>,
    pub font: Option<Var<String>>,
    pub size: Option<Var<f32>>,
}

#[typetag::deserialize(name = "Text")]
impl WidgetBuilder for TextBuilder {
    fn combine(&mut self, rhs: &dyn WidgetBuilder) {
        if let Some(other) = rhs.as_any().downcast_ref::<Self>() {
            if let Some(s) = &other.text {
                self.text.get_or_insert_with(|| s.clone());
            }
            if let Some(s) = &other.colour {
                self.colour.get_or_insert_with(|| s.clone());
            }
            if let Some(s) = &other.font {
                self.font.get_or_insert_with(|| s.clone());
            }
            if let Some(s) = &other.size {
                self.size.get_or_insert_with(|| s.clone());
            }
        }
    }

    fn create_widget(&self, stream: &mut TokenStream) {
        let text = match &self.text {
            Some(Var::Value(v)) => v.to_token_stream(),
            _ => "".to_token_stream(),
        };

        stream.extend(quote! {
            ::gui_widget::Text::new(String::from(#text))
        });
    }
    fn on_var_update(&self, widget: Ident, var: &str, value: Ident, stream: &mut TokenStream) {
        match &self.text {
            Some(Var::Variable(v)) if v == var => stream.extend(quote! {#widget::set_text(#value)}),
            _ => {}
        }
    }
}
