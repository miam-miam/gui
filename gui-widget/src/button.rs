use gui_core::glazier::kurbo::{Shape, Size};
use gui_core::layout::LayoutConstraints;
use gui_core::parse::fluent::Fluent;
use gui_core::parse::WidgetDeclaration;
use gui_core::vello::kurbo::{Affine, Vec2};
use gui_core::vello::peniko::{BlendMode, Brush, Color, Compose, Fill, Mix, Stroke};
use gui_core::vello::SceneFragment;
use gui_core::widget::{RenderHandle, ResizeHandle, Widget, WidgetBuilder, WidgetEvent, WidgetID};
use gui_core::{widget, Colour, SceneBuilder, ToComponent, ToHandler, Var};
use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use serde::Deserialize;
use std::marker::PhantomData;
use widget::EventHandle;

pub trait ButtonHandler<T: ToHandler<BaseHandler = Self>> {
    fn on_press(&mut self) {}
}

pub struct Button<T: ToHandler<BaseHandler = C>, C: ToComponent, W: Widget<C>> {
    id: WidgetID,
    background_colour: Colour,
    disabled_colour: Colour,
    clicked_colour: Colour,
    hover_colour: Colour,
    border_colour: Colour,
    disabled: bool,
    child: W,
    phantom: PhantomData<(T, C)>,
}

impl<T: ToHandler<BaseHandler = C>, C: ToComponent, W: Widget<C>> Button<T, C, W> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: WidgetID,
        background_colour: Colour,
        disabled_colour: Colour,
        clicked_colour: Colour,
        hover_colour: Colour,
        border_colour: Colour,
        disabled: bool,
        child: W,
    ) -> Self {
        Button {
            id,
            background_colour,
            disabled_colour,
            clicked_colour,
            hover_colour,
            border_colour,
            disabled,
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

const STOKE_WIDTH: f64 = 0.58;

impl<T: ToHandler<BaseHandler = C>, C: ToComponent + ButtonHandler<T>, W: Widget<C>> Widget<C>
    for Button<T, C, W>
{
    fn id(&self) -> WidgetID {
        self.id
    }

    fn render(&mut self, scene: &mut SceneBuilder, handle: &mut RenderHandle<C>) {
        let clicking = handle.is_active(self.id());
        let hovered = handle.is_hovered(self.id());
        let affine = if handle.is_active(self.id()) {
            Affine::translate(Vec2::new(0.0, 0.875))
        } else {
            Affine::IDENTITY
        };
        let fill_colour = if self.disabled {
            self.disabled_colour
        } else if clicking && hovered {
            self.clicked_colour
        } else if hovered {
            self.hover_colour
        } else {
            self.background_colour
        };
        let rect = handle
            .get_local_rect(self.id())
            .inset(-0.5 * STOKE_WIDTH)
            .to_rounded_rect(4.0);
        scene.fill(
            Fill::NonZero,
            affine,
            &Brush::Solid(fill_colour.0),
            None,
            &rect,
        );

        let mut fragment = SceneFragment::new();
        let mut builder = SceneBuilder::for_fragment(&mut fragment);
        handle.render_widgets(&mut builder, [&mut self.child].into_iter());

        scene.append(
            &fragment,
            clicking.then(|| Affine::translate(Vec2::new(0.0, 0.875))),
        );

        if self.disabled {
            scene.push_layer(
                BlendMode::new(Mix::Screen, Compose::SrcOver),
                1.0,
                Affine::IDENTITY,
                &rect,
            );

            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                &Brush::Solid(Color::GRAY),
                None,
                &rect,
            );

            scene.pop_layer()
        } else {
            scene.stroke(
                &Stroke::new(STOKE_WIDTH as f32),
                affine,
                &Brush::Solid(self.border_colour.0),
                None,
                &handle.get_local_rect(self.id()).to_rounded_rect(4.5),
            );
        }
    }

    fn resize(&mut self, mut constraints: LayoutConstraints, handle: &mut ResizeHandle<C>) -> Size {
        let padding = Size::new(STOKE_WIDTH + 18.0, STOKE_WIDTH);
        constraints = constraints.deset(padding);
        let mut child_size = handle.layout_widget(
            padding.to_vec2().to_point(),
            &mut self.child,
            constraints.min_clamp(Size::new(0.0, 18.0)),
        );
        child_size += padding * 2.0;
        child_size
    }

    fn event(&mut self, event: WidgetEvent, handle: &mut EventHandle<C>) {
        if self.disabled {
            return;
        }
        let hit = event.get_point().map_or(false, |pos| {
            handle
                .get_global_rect(self.id())
                .to_rounded_rect(4.0)
                .contains(pos)
        });
        match event {
            WidgetEvent::PointerUp(_) => {
                if hit {
                    handle.set_active(self.id(), false);
                    handle.invalidate_id(self.id());
                    handle.get_handler().on_press();
                }
            }
            WidgetEvent::PointerDown(_) => {
                if hit {
                    handle.set_active(self.id(), true);
                    handle.invalidate_id(self.id());
                }
            }
            WidgetEvent::PointerMove(_) => {
                if hit && handle.add_hover(self.id()) {
                    handle.invalidate_id(self.id());
                }
            }
            WidgetEvent::HoverChange | WidgetEvent::ActiveChange => handle.invalidate_id(self.id()),
        }
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
        component: &Ident,
        widget: Option<&TokenStream>,
        stream: &mut TokenStream,
    ) {
        stream.extend(quote!(::gui::gui_widget::Button<#handler, #component, #widget>));
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

    fn create_widget(&self, id: WidgetID, widget: Option<&TokenStream>, stream: &mut TokenStream) {
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
            ::gui::gui_widget::Button::new(#id, #background_colour, #disabled_colour, #clicked_colour, #hover_colour, #border_colour, #disabled, #widget)
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
            "disabled" => stream.extend(quote! {#widget.set_disabled(#value);}),
            "background_colour" => stream.extend(quote! {#widget.set_background_colour(#value);}),
            "disabled_colour" => stream.extend(quote! {#widget.set_disabled_colour(#value);}),
            "clicked_colour" => stream.extend(quote! {#widget.set_clicked_colour(#value);}),
            "hover_colour" => stream.extend(quote! {#widget.set_hover_colour(#value);}),
            "border_colour" => stream.extend(quote! {#widget.set_border_colour(#value);}),
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

    fn get_widgets(&mut self) -> Option<Vec<&mut WidgetDeclaration>> {
        Some(self.child.iter_mut().collect())
    }

    fn widgets(&self) -> Option<Vec<(TokenStream, &WidgetDeclaration)>> {
        Some(
            self.child
                .iter()
                .map(|c| (quote!(.get_widget()), c))
                .collect(),
        )
    }
}
