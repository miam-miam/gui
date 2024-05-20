use std::marker::PhantomData;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use serde::Deserialize;

use gui_core::{Colour, SceneBuilder, ToComponent, ToHandler, Var, widget};
use gui_core::{Children, MutWidgetChildren, WidgetChildren};
use gui_core::glazier::Cursor;
use gui_core::glazier::kurbo::{Shape, Size};
use gui_core::layout::LayoutConstraints;
use gui_core::parse::var::Name;
use gui_core::parse::WidgetDeclaration;
use gui_core::vello::kurbo::{Affine, Vec2};
use gui_core::vello::peniko::{BlendMode, Brush, Color, Compose, Fill, Mix, Stroke};
use gui_core::vello::SceneFragment;
use gui_core::widget::{
    RenderHandle, ResizeHandle, UpdateHandle, Widget, WidgetBuilder, WidgetEvent, WidgetID,
};
use gui_derive::WidgetBuilder;
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
    child: Option<W>,
    phantom: PhantomData<(T, C)>,
}

impl<T: ToHandler<BaseHandler = C>, C: ToComponent, W: Widget<C>> Button<T, C, W> {
    pub fn new(id: WidgetID) -> Self {
        Button {
            id,
            background_colour: Default::default(),
            disabled_colour: Default::default(),
            clicked_colour: Default::default(),
            hover_colour: Default::default(),
            border_colour: Default::default(),
            disabled: Default::default(),
            child: None,
            phantom: PhantomData,
        }
    }

    pub fn set_disabled(&mut self, disabled: bool, handle: &mut UpdateHandle) {
        self.disabled = disabled;
        handle.invalidate_id(self.id)
    }
    pub fn set_background_colour(&mut self, colour: Colour, handle: &mut UpdateHandle) {
        self.background_colour = colour;
        handle.invalidate_id(self.id)
    }
    pub fn set_disabled_colour(&mut self, colour: Colour, handle: &mut UpdateHandle) {
        self.disabled_colour = colour;
        handle.invalidate_id(self.id)
    }
    pub fn set_clicked_colour(&mut self, colour: Colour, handle: &mut UpdateHandle) {
        self.clicked_colour = colour;
        handle.invalidate_id(self.id)
    }
    pub fn set_hover_colour(&mut self, colour: Colour, handle: &mut UpdateHandle) {
        self.hover_colour = colour;
        handle.invalidate_id(self.id)
    }
    pub fn set_border_colour(&mut self, colour: Colour, handle: &mut UpdateHandle) {
        self.border_colour = colour;
        handle.invalidate_id(self.id)
    }
    pub fn get_widget(&mut self) -> &mut Option<W> {
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
        let clicking = handle.is_active(self.id);
        let hovered = handle.is_hovered(self.id);
        let affine = if handle.is_active(self.id) {
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
            .get_local_rect(self.id)
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
        handle.render_widgets(&mut builder, [self.child.as_mut().unwrap()].into_iter());

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
                &handle.get_local_rect(self.id).to_rounded_rect(4.5),
            );
        }
    }

    fn resize(&mut self, mut constraints: LayoutConstraints, handle: &mut ResizeHandle<C>) -> Size {
        let padding = Size::new(STOKE_WIDTH + 18.0, STOKE_WIDTH);
        constraints = constraints.deset(padding);
        let mut child_size = handle.layout_widget(
            padding.to_vec2().to_point(),
            self.child.as_mut().unwrap(),
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
                .get_global_rect(self.id)
                .to_rounded_rect(4.0)
                .contains(pos)
        });
        match event {
            WidgetEvent::PointerUp(_) => {
                handle.set_active(self.id, false);
                handle.invalidate_id(self.id);
                if hit {
                    handle.get_handler().on_press();
                }
            }
            WidgetEvent::PointerDown(_) => {
                if hit {
                    handle.set_active(self.id, true);
                    handle.invalidate_id(self.id);
                }
            }
            WidgetEvent::PointerMove(_) => {
                if hit {
                    handle.set_cursor(&Cursor::Pointer);
                    if handle.add_hover(self.id) {
                        handle.invalidate_id(self.id);
                    }
                }
            }
            WidgetEvent::HoverChange | WidgetEvent::ActiveChange => handle.invalidate_id(self.id),
        }
    }
}

#[derive(Deserialize, WidgetBuilder, Debug, Clone)]
#[serde(deny_unknown_fields)]
#[widget(
name = "Button",
type_path = "::gui::gui_widget::Button<#handler, #component, #child>",
init_path = "new"
)]
pub struct ButtonBuilder {
    #[widget(property = "set_disabled")]
    disabled: Option<Var<bool>>,
    #[widget(property = "set_background_colour")]
    #[widget(default = Colour(Color::WHITE))]
    background_colour: Option<Var<Colour>>,
    #[widget(property = "set_disabled_colour")]
    #[widget(default = Colour(Color::rgb8(241, 243, 245)))]
    disabled_colour: Option<Var<Colour>>,
    #[widget(property = "set_clicked_colour")]
    #[widget(default = Colour(Color::rgb8(248, 249, 250)))]
    clicked_colour: Option<Var<Colour>>,
    #[widget(property = "set_hover_colour")]
    #[widget(default = Colour(Color::rgb8(248, 249, 250)))]
    hover_colour: Option<Var<Colour>>,
    #[widget(property = "set_border_colour")]
    #[widget(default = Colour(Color::rgb8(206, 212, 218)))]
    border_colour: Option<Var<Colour>>,
    #[widget(child = "get_widget")]
    child: Option<WidgetDeclaration>,
}
