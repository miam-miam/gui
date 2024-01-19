use gui_core::parse::fluent::Fluent;
use gui_core::parse::WidgetDeclaration;
use gui_core::vello::kurbo::{Affine, Rect};
use gui_core::vello::peniko::{Brush, Color, Fill, Stroke};
use gui_core::widget::{Widget, WidgetBuilder};
use gui_core::{Colour, FontContext, SceneBuilder, Var};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use serde::Deserialize;

pub struct Button {
    background_colour: Colour,
    // disabled_colour: Colour,
    // clicked_colour: Colour,
    // hover_colour: Colour,
    disabled: bool,
    size: Option<Rect>,
}

impl Button {
    pub fn new(
        background_colour: Colour,
        // disabled_colour: Colour,
        // clicked_colour: Colour,
        // hover_colour: Colour,
        disabled: bool,
    ) -> Self {
        Button {
            background_colour,
            // disabled_colour,
            // clicked_colour,
            // hover_colour,
            disabled,
            size: Some(Rect::new(0.0, 0.0, 100.0, 40.0)),
        }
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    pub fn set_background_colour(&mut self, colour: Colour) {
        self.background_colour = colour;
    }
}

impl Widget for Button {
    fn render(&mut self, mut scene: SceneBuilder, fcx: &mut FontContext) {
        if let Some(size) = self.size {
            let rect = size.inset(-1.0).to_rounded_rect(4.5);
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(self.background_colour.0),
                None,
                &rect,
            );
            scene.stroke(
                &Stroke::new(2.0),
                Affine::IDENTITY,
                &Brush::Solid(Color::BLACK),
                None,
                &size.to_rounded_rect(4.5),
            );
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ButtonBuilder {
    pub disabled: Option<Var<bool>>,
    pub background_colour: Option<Var<Colour>>,
    pub child: Option<WidgetDeclaration>,
}

#[typetag::deserialize(name = "Button")]
impl WidgetBuilder for ButtonBuilder {
    fn widget_type(&self, stream: &mut TokenStream) {
        stream.extend(quote!(::gui::gui_widget::Button));
    }

    fn name(&self) -> &'static str {
        "Button"
    }
    fn combine(&mut self, rhs: &dyn WidgetBuilder) {
        if let Some(other) = rhs.as_any().downcast_ref::<Self>() {
            if let Some(s) = &other.disabled {
                self.disabled.get_or_insert_with(|| s.clone());
            }
            if let Some(s) = &other.background_colour {
                self.background_colour.get_or_insert_with(|| s.clone());
            }
        }
    }

    fn create_widget(&self, stream: &mut TokenStream) {
        let disabled = match &self.disabled {
            Some(Var::Value(v)) => v.to_token_stream(),
            _ => false.to_token_stream(),
        };

        let background_colour = match &self.background_colour {
            Some(Var::Value(v)) => v.to_token_stream(),
            _ => Colour(Color::LIGHT_GRAY).to_token_stream(),
        };

        stream.extend(quote! {
            ::gui::gui_widget::Button::new(#background_colour, #disabled)
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
            "disabled" => stream.extend(quote! {#widget.set_disabled(#value)}),
            "background_colour" => stream.extend(quote! {#widget.set_background_colour(#value)}),
            _ => {}
        }
    }

    fn get_fluents(&self) -> Vec<(&'static str, &Fluent)> {
        vec![]
    }

    fn get_vars(&self) -> Vec<(&'static str, &str)> {
        let mut array = vec![];
        if let Some(Var::Variable(v)) = &self.disabled {
            array.push(("disabled", v.as_str()));
        }
        if let Some(Var::Variable(v)) = &self.background_colour {
            array.push(("background_colour", v.as_str()));
        }
        array
    }
}
