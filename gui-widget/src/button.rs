use gui_core::glazier::kurbo::Shape;
use gui_core::glazier::{PointerEvent, WindowHandle};
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
    fn get(base: &mut T::BaseHandler) -> &mut Self {
        base
    }
    fn on_press(&mut self) {}
}

pub struct Button<T: ToHandler<BaseHandler = H>, H: ButtonHandler<T>, W: Widget<H>> {
    background_colour: Colour,
    disabled_colour: Colour,
    clicked_colour: Colour,
    hover_colour: Colour,
    border_colour: Colour,
    disabled: bool,
    size: Rect,
    hovered: bool,
    clicking: bool,
    child: W,
    phantom: PhantomData<(T, H)>,
}

impl<T: ToHandler<BaseHandler = H>, H: ButtonHandler<T>, W: Widget<H>> Button<T, H, W> {
    pub fn new(
        background_colour: Colour,
        disabled_colour: Colour,
        clicked_colour: Colour,
        hover_colour: Colour,
        border_colour: Colour,
        disabled: bool,
        child: W,
    ) -> Self {
        Button {
            background_colour,
            disabled_colour,
            clicked_colour,
            hover_colour,
            border_colour,
            disabled,
            size: Rect::new(0.0, 0.0, 150.0, 40.0),
            hovered: false,
            clicking: false,
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
    pub fn set_disabled_colour(&mut self, colour: Colour) {
        self.disabled_colour = colour;
    }
    pub fn set_clicked_colour(&mut self, colour: Colour) {
        self.clicked_colour = colour;
    }
    pub fn set_hover_colour(&mut self, colour: Colour) {
        self.hover_colour = colour;
    }
    pub fn set_border_colour(&mut self, colour: Colour) {
        self.border_colour = colour;
    }
    pub fn get_widget(&mut self) -> &mut W {
        &mut self.child
    }
}

impl<T: ToHandler<BaseHandler = H>, H: ButtonHandler<T>, W: Widget<H>> Widget<H>
    for Button<T, H, W>
{
    fn render(&mut self, mut scene: SceneBuilder, fcx: &mut FontContext) {
        let stroke_width = 0.58_f32;
        let affine = if self.clicking {
            Affine::translate(Vec2::new(0.0, 0.875))
        } else {
            Affine::IDENTITY
        };
        let fill_colour = if self.clicking && self.hovered {
            self.clicked_colour
        } else if self.hovered {
            self.hover_colour
        } else {
            self.background_colour
        };
        let rect = self
            .size
            .inset(-0.5 * stroke_width as f64)
            .to_rounded_rect(4.0);
        scene.fill(
            Fill::NonZero,
            affine,
            &Brush::Solid(fill_colour.0),
            None,
            &rect,
        );
        let mut fragment = SceneFragment::new();
        self.child
            .render(SceneBuilder::for_fragment(&mut fragment), fcx);
        scene.append(
            &fragment,
            Some(Affine::translate(Vec2::new(
                stroke_width as f64 + 18.0,
                stroke_width as f64 + if self.clicking { 0.875 } else { 0.0 },
            ))),
        );
        scene.stroke(
            &Stroke::new(stroke_width),
            affine,
            &Brush::Solid(self.border_colour.0),
            None,
            &self.size.to_rounded_rect(4.5),
        );
    }

    fn pointer_down(&mut self, event: &PointerEvent, _window: &WindowHandle, _handler: &mut H) {
        self.clicking = self.size.to_rounded_rect(4.0).contains(event.pos);
    }

    fn pointer_up(&mut self, _event: &PointerEvent, _window: &WindowHandle, handler: &mut H) {
        self.clicking = false;
        if self.hovered {
            handler.on_press();
        }
    }

    fn pointer_move(&mut self, event: &PointerEvent, _window: &WindowHandle, _handler: &mut H) {
        self.hovered = self.size.to_rounded_rect(4.0).contains(event.pos);
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ButtonBuilder {
    disabled: Option<Var<bool>>,
    background_colour: Option<Var<Colour>>,
    disabled_colour: Option<Var<Colour>>,
    clicked_colour: Option<Var<Colour>>,
    hover_colour: Option<Var<Colour>>,
    border_colour: Option<Var<Colour>>,
    child: Option<WidgetDeclaration>,
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
            if let Some(s) = &other.disabled_colour {
                self.disabled_colour.get_or_insert_with(|| s.clone());
            }
            if let Some(s) = &other.clicked_colour {
                self.clicked_colour.get_or_insert_with(|| s.clone());
            }
            if let Some(s) = &other.hover_colour {
                self.hover_colour.get_or_insert_with(|| s.clone());
            }
            if let Some(s) = &other.border_colour {
                self.border_colour.get_or_insert_with(|| s.clone());
            }
        }
    }

    fn create_widget(&self, widget: Option<&TokenStream>, stream: &mut TokenStream) {
        let background_colour = match &self.background_colour {
            Some(Var::Value(v)) => v.to_token_stream(),
            _ => Colour(Color::WHITE).to_token_stream(),
        };
        let disabled_colour = match &self.disabled_colour {
            Some(Var::Value(v)) => v.to_token_stream(),
            _ => Colour(Color::rgb8(241, 243, 245)).to_token_stream(),
        };
        let clicked_colour = match &self.clicked_colour {
            Some(Var::Value(v)) => v.to_token_stream(),
            _ => Colour(Color::rgb8(248, 249, 250)).to_token_stream(),
        };
        let hover_colour = match &self.hover_colour {
            Some(Var::Value(v)) => v.to_token_stream(),
            _ => Colour(Color::rgb8(248, 249, 250)).to_token_stream(),
        };
        let border_colour = match &self.border_colour {
            Some(Var::Value(v)) => v.to_token_stream(),
            _ => Colour(Color::rgb8(206, 212, 218)).to_token_stream(),
        };
        let disabled = match &self.disabled {
            Some(Var::Value(v)) => v.to_token_stream(),
            _ => false.to_token_stream(),
        };

        stream.extend(quote! {
            ::gui::gui_widget::Button::new(#background_colour, #disabled_colour, #clicked_colour, #hover_colour, #border_colour, #disabled, #widget)
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
            "disabled_colour" => stream.extend(quote! {#widget.set_disabled_colour(#value)}),
            "clicked_colour" => stream.extend(quote! {#widget.set_clicked_colour(#value)}),
            "hover_colour" => stream.extend(quote! {#widget.set_hover_colour(#value)}),
            "border_colour" => stream.extend(quote! {#widget.set_border_colour(#value)}),
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
        if let Some(Var::Variable(v)) = &self.disabled_colour {
            array.push(("disabled_colour", v.as_str()));
        }
        if let Some(Var::Variable(v)) = &self.clicked_colour {
            array.push(("clicked_colour", v.as_str()));
        }
        if let Some(Var::Variable(v)) = &self.hover_colour {
            array.push(("hover_colour", v.as_str()));
        }
        if let Some(Var::Variable(v)) = &self.border_colour {
            array.push(("border_colour", v.as_str()));
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
