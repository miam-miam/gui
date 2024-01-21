use gui_core::common::text;
use gui_core::common::text::ParleyBrush;
use gui_core::parley::layout::Layout;
use gui_core::parley::style::{FontWeight, StyleProperty};
use gui_core::parley::{layout, LayoutContext};
use gui_core::parse::fluent::Fluent;
use gui_core::parse::WidgetDeclaration;
use gui_core::vello::kurbo::Affine;
use gui_core::vello::peniko::{Brush, Color};
use gui_core::widget::{Widget, WidgetBuilder};
use gui_core::{Colour, FontContext, SceneBuilder, Var};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use serde::Deserialize;
use std::borrow::Cow;

pub struct Text {
    text: String,
    colour: Colour,
    size: f32,
    layout: Option<Layout<ParleyBrush>>,
}

impl Text {
    pub fn new(text: String, colour: Colour, size: f32) -> Self {
        Text {
            text,
            colour,
            size,
            layout: None,
        }
    }

    fn build(&mut self, fcx: &mut FontContext) {
        let mut lcx = LayoutContext::new();
        let mut layout_builder = lcx.ranged_builder(fcx, &self.text, 1.0);
        layout_builder.push_default(&StyleProperty::FontSize(self.size));
        layout_builder.push_default(&StyleProperty::FontWeight(FontWeight::SEMI_BOLD));
        layout_builder.push_default(&StyleProperty::Brush(ParleyBrush(Brush::Solid(
            self.colour.0,
        ))));
        self.layout = Some(layout_builder.build());
    }

    pub fn set_text(&mut self, text: Cow<'_, str>) {
        if self.text != text {
            self.text = text.into_owned();
            self.layout = None;
        }
    }

    pub fn set_colour(&mut self, colour: Colour) {
        if self.colour != colour {
            self.colour = colour;
            self.layout = None;
        }
    }

    pub fn set_size(&mut self, size: f32) {
        if self.size != size {
            self.size = size;
            self.layout = None;
        }
    }
}

impl<T> Widget<T> for Text {
    fn render(&mut self, mut scene: SceneBuilder, fcx: &mut FontContext) {
        if self.layout.is_none() {
            if self.text.is_empty() {
                return;
            }
            self.build(fcx);
        }

        let layout = self.layout.as_mut().unwrap();
        layout.break_all_lines(None, layout::Alignment::Start);
        text::render_text(&mut scene, Affine::translate((0.0, 0.0)), layout);
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct TextBuilder {
    pub text: Option<Fluent>,
    pub colour: Option<Var<Colour>>,
    pub size: Option<Var<f32>>,
}

#[typetag::deserialize(name = "Text")]
impl WidgetBuilder for TextBuilder {
    fn widget_type(
        &self,
        _handler: Option<&Ident>,
        _comp_struct: &Ident,
        _widget: Option<&TokenStream>,
        stream: &mut TokenStream,
    ) {
        stream.extend(quote!(::gui::gui_widget::Text));
    }

    fn name(&self) -> &'static str {
        "Text"
    }
    fn combine(&mut self, rhs: &dyn WidgetBuilder) {
        if let Some(other) = rhs.as_any().downcast_ref::<Self>() {
            if let Some(s) = &other.text {
                self.text.get_or_insert_with(|| s.clone());
            }
            if let Some(s) = &other.colour {
                self.colour.get_or_insert_with(|| s.clone());
            }
            if let Some(s) = &other.size {
                self.size.get_or_insert_with(|| s.clone());
            }
        }
    }

    fn create_widget(&self, _widget: Option<&TokenStream>, stream: &mut TokenStream) {
        let colour = match &self.colour {
            Some(Var::Value(v)) => v.to_token_stream(),
            _ => Colour(Color::rgb8(33, 37, 41)).to_token_stream(),
        };
        let size = match &self.size {
            Some(Var::Value(v)) => v.to_token_stream(),
            _ => 14.0f32.to_token_stream(),
        };

        stream.extend(quote! {
            ::gui::gui_widget::Text::new(String::new(), #colour, #size)
        });
    }

    fn on_property_update(
        &self,
        property: &'static str,
        widget: &Ident,
        value: &Ident,
        stream: &mut TokenStream,
    ) {
        match property {
            "text" => stream.extend(quote! {#widget.set_text(#value);}),
            "colour" => stream.extend(quote! {#widget.set_colour(#value);}),
            "size" => stream.extend(quote! {#widget.set_size(#value);}),
            _ => {}
        }
    }

    fn get_fluents(&self) -> Vec<(&'static str, &Fluent)> {
        self.text.iter().map(|f| ("text", f)).collect()
    }

    fn get_vars(&self) -> Vec<(&'static str, &str)> {
        let mut array = vec![];
        if let Some(Var::Variable(v)) = &self.colour {
            array.push(("colour", v.as_str()));
        }
        if let Some(Var::Variable(v)) = &self.size {
            array.push(("size", v.as_str()));
        }
        array
    }

    fn has_handler(&self) -> bool {
        false
    }

    fn get_widgets(&self) -> Vec<&Option<WidgetDeclaration>> {
        vec![]
    }
}
