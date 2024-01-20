use gui_core::parse::fluent::Fluent;
use gui_core::parse::WidgetDeclaration;
use gui_core::vello::kurbo::{Affine, Rect, Vec2};
use gui_core::vello::peniko::{Brush, Color, Fill, Stroke};
use gui_core::vello::SceneFragment;
use gui_core::widget::{Widget, WidgetBuilder};
use gui_core::{Colour, FontContext, SceneBuilder, ToHandler, Var};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use serde::Deserialize;
use std::marker::PhantomData;

pub trait ButtonHandler<T: ToHandler<BaseHandler = Self>> {
    // Does not need to be overridden
    fn get(base: &T::BaseHandler) -> &Self {
        base
    }
    fn on_press(&self) {}
}

pub struct Button<T: ToHandler<BaseHandler = H>, H: ButtonHandler<T>, W: Widget<H>> {
    background_colour: Colour,
    // disabled_colour: Colour,
    // clicked_colour: Colour,
    // hover_colour: Colour,
    disabled: bool,
    size: Option<Rect>,
    child: W,
    phantom: PhantomData<(T, H)>,
}

impl<T: ToHandler<BaseHandler = H>, H: ButtonHandler<T>, W: Widget<H>> Button<T, H, W> {
    pub fn new(
        background_colour: Colour,
        // disabled_colour: Colour,
        // clicked_colour: Colour,
        // hover_colour: Colour,
        disabled: bool,
        child: W,
    ) -> Self {
        Button {
            background_colour,
            // disabled_colour,
            // clicked_colour,
            // hover_colour,
            disabled,
            size: Some(Rect::new(0.0, 0.0, 100.0, 40.0)),
            child,
            phantom: PhantomData,
        }
    }

    pub fn set_disabled(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    pub fn set_background_colour(&mut self, colour: Colour) {
        self.background_colour = colour;
    }
    
    pub fn get_widget(&mut self) -> &mut W {
        &mut self.child
    }
}

impl<T: ToHandler<BaseHandler = H>, H: ButtonHandler<T>, W: Widget<H>> Widget<H>
    for Button<T, H, W>
{
    fn render(&mut self, mut scene: SceneBuilder, fcx: &mut FontContext) {
        if let Some(size) = self.size {
            let stroke_width = 2.0_f32;
            let rect = size.inset(-0.5 * stroke_width as f64).to_rounded_rect(4.5);
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(self.background_colour.0),
                None,
                &rect,
            );
            let mut fragment = SceneFragment::new();
            self.child
                .render(SceneBuilder::for_fragment(&mut fragment), fcx);
            scene.append(
                &fragment,
                Some(Affine::translate(Vec2::new(
                    stroke_width as f64,
                    stroke_width as f64,
                ))),
            );
            scene.stroke(
                &Stroke::new(stroke_width),
                Affine::IDENTITY,
                &Brush::Solid(Color::BLACK),
                None,
                &size.to_rounded_rect(4.5),
            );
        }
    }

    fn on_press(&mut self, handler: &mut H) {
        H::get(handler).on_press();
        self.child.on_press(handler);
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
    fn widget_type(
        &self,
        handler: Option<&Ident>,
        comp_struct: &Ident,
        widget: Option<&TokenStream>,
        stream: &mut TokenStream,
    ) {
        stream.extend(quote!(::gui::gui_widget::Button<#handler, #comp_struct, #widget>));
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

    fn create_widget(&self, widget: Option<&TokenStream>, stream: &mut TokenStream) {
        let disabled = match &self.disabled {
            Some(Var::Value(v)) => v.to_token_stream(),
            _ => false.to_token_stream(),
        };

        let background_colour = match &self.background_colour {
            Some(Var::Value(v)) => v.to_token_stream(),
            _ => Colour(Color::LIGHT_GRAY).to_token_stream(),
        };

        stream.extend(quote! {
            ::gui::gui_widget::Button::new(#background_colour, #disabled, #widget)
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

    fn has_handler(&self) -> bool {
        true
    }

    fn get_widgets(&self) -> Vec<&Option<WidgetDeclaration>> {
        vec![&self.child]
    }
}
