use std::borrow::Cow;

use serde::Deserialize;

use gui_custom::common::text;
use gui_custom::common::text::ParleyBrush;
use gui_custom::glazier::kurbo::Size;
use gui_custom::layout::LayoutConstraints;
use gui_custom::parley::layout::{Alignment, Layout};
use gui_custom::parley::style::{FontWeight, StyleProperty};
use gui_custom::parley::LayoutContext;
use gui_custom::vello::kurbo::Affine;
use gui_custom::vello::peniko::{Brush, Color};
use gui_custom::widget::{
    EventHandle, RenderHandle, ResizeHandle, UpdateHandle, Widget, WidgetEvent, WidgetID,
};
use gui_custom::WidgetBuilder;
use gui_custom::{Colour, Fluent, FontContext, SceneBuilder, ToComponent, Var};

pub struct Text {
    id: WidgetID,
    text: String,
    colour: Colour,
    size: f32,
    // Use box to reduce struct size
    layout: Option<Box<Layout<ParleyBrush>>>,
}

impl Text {
    pub fn new(id: WidgetID) -> Self {
        Text {
            id,
            text: Default::default(),
            colour: Default::default(),
            size: Default::default(),
            layout: None,
        }
    }

    fn build(&mut self, fcx: &mut FontContext) {
        let mut lcx = LayoutContext::new();
        let mut layout_builder = lcx.ranged_builder(fcx, &self.text, 1.0);
        layout_builder.push_default(&StyleProperty::FontSize(self.size));
        layout_builder.push_default(&StyleProperty::FontWeight(FontWeight::BOLD));
        layout_builder.push_default(&StyleProperty::Brush(ParleyBrush(Brush::Solid(
            self.colour.0,
        ))));
        self.layout = Some(Box::new(layout_builder.build()));
    }

    pub fn set_text(&mut self, text: Cow<'_, str>, handle: &mut UpdateHandle) {
        if self.text != text {
            self.text = text.into_owned();
            self.layout = None;
            handle.resize();
        }
    }

    pub fn set_colour(&mut self, colour: Colour, handle: &mut UpdateHandle) {
        if self.colour != colour {
            self.colour = colour;
            self.layout = None;
            handle.invalidate_id(self.id);
        }
    }

    pub fn set_size(&mut self, size: f32, handle: &mut UpdateHandle) {
        if self.size != size {
            self.size = size;
            self.layout = None;
            handle.resize();
        }
    }
}

impl<C: ToComponent> Widget<C> for Text {
    fn id(&self) -> WidgetID {
        self.id
    }

    fn render(&mut self, scene: &mut SceneBuilder, handle: &mut RenderHandle<C>) {
        if self.layout.is_none() {
            if self.text.is_empty() {
                return;
            }
            self.build(handle.get_fcx());
        }

        let layout = self.layout.as_mut().unwrap();
        text::render_text(scene, Affine::IDENTITY, layout);
    }

    fn resize(&mut self, constraints: LayoutConstraints, handle: &mut ResizeHandle<C>) -> Size {
        if self.layout.is_none() {
            if self.text.is_empty() {
                return constraints.get_min();
            }
            self.build(handle.get_fcx());
        }
        let layout = self.layout.as_mut().unwrap();
        layout.break_all_lines(constraints.max_advance(), Alignment::Start);
        Size::new(layout.width() as f64, layout.height() as f64)
    }

    fn event(&mut self, _event: WidgetEvent, _handle: &mut EventHandle<C>) {}
}

#[derive(Deserialize, WidgetBuilder, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
#[widget(
    name = "Text",
    type_path = "::gui::gui_widget::Text",
    init_path = "new"
)]
pub struct TextBuilder {
    #[widget(fluent = "set_text")]
    pub text: Option<Fluent>,
    #[widget(property = "set_colour")]
    #[widget(default = Colour(Color::rgb8(33, 37, 41)))]
    pub colour: Option<Var<Colour>>,
    #[widget(property = "set_size", default = 14.0f32)]
    pub size: Option<Var<f32>>,
}
