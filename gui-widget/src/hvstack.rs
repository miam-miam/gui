use itertools::Itertools;
use serde::Deserialize;

use gui_core::glazier::kurbo::Rect;
use gui_core::parse::WidgetDeclaration;
use gui_core::widget::{
    EventHandle, RenderHandle, ResizeHandle, UpdateHandle, Widget, WidgetEvent, WidgetID,
};
use gui_core::{LayoutConstraints, Point, SceneBuilder, Size, ToComponent, Var};
use gui_derive::WidgetBuilder;

enum Axis {
    Horizontal,
    Vertical,
}

impl Axis {
    fn to_size(&self, value: f64) -> Size {
        match self {
            Axis::Horizontal => Size::new(value, 0.0),
            Axis::Vertical => Size::new(0.0, value),
        }
    }

    fn get_axis(&self, size: Size) -> f64 {
        match self {
            Axis::Horizontal => size.width,
            Axis::Vertical => size.height,
        }
    }

    fn invert(&self) -> Self {
        match self {
            Axis::Horizontal => Axis::Vertical,
            Axis::Vertical => Axis::Horizontal,
        }
    }
}

pub struct HVStack<W> {
    id: WidgetID,
    axis: Axis,
    spacing: f32,
    children: Vec<W>,
}

impl<W> HVStack<W> {
    pub fn new_horizontal(id: WidgetID) -> Self {
        Self {
            id,
            axis: Axis::Horizontal,
            spacing: Default::default(),
            children: vec![],
        }
    }

    pub fn new_vertical(id: WidgetID) -> Self {
        Self {
            id,
            axis: Axis::Vertical,
            spacing: Default::default(),
            children: vec![],
        }
    }

    pub fn set_spacing(&mut self, spacing: f32, handle: &mut UpdateHandle) {
        self.spacing = spacing;
        handle.invalidate_id(self.id);
    }

    pub fn widgets(&mut self) -> &mut Vec<W> {
        &mut self.children
    }
}

impl<C: ToComponent, W: Widget<C>> Widget<C> for HVStack<W> {
    fn id(&self) -> WidgetID {
        self.id
    }

    fn render(&mut self, scene: &mut SceneBuilder, handle: &mut RenderHandle<C>) {
        handle.render_widgets(scene, self.children.iter_mut());
    }
    fn resize(&mut self, constraints: LayoutConstraints, handle: &mut ResizeHandle<C>) -> Size {
        let child_length = self.children.len();
        let total_spacing = self.spacing as f64 * (child_length - 1) as f64;
        let mut remaining = constraints.map(|s| s - self.axis.to_size(total_spacing));

        let layouts =
            self.children
                .iter_mut()
                .enumerate()
                .map(|(i, child)| {
                    let allocated_space = match self.axis {
                        Axis::Horizontal => remaining
                            .map(|s| Size::new(s.width / (child_length - i) as f64, s.height)),
                        Axis::Vertical => remaining
                            .map(|s| Size::new(s.width, s.height / (child_length - i) as f64)),
                    };
                    let size = child.resize(allocated_space, handle);
                    remaining = remaining.map(|s| s - self.axis.to_size(self.axis.get_axis(size)));
                    (size, child.id())
                })
                .collect_vec();

        let max_length = layouts
            .iter()
            .map(|(s, _)| self.axis.invert().get_axis(*s))
            .reduce(f64::max)
            .unwrap_or_default();
        let mut acc = 0.0;

        for (s, id) in layouts.iter().copied() {
            let pos = match self.axis {
                Axis::Horizontal => {
                    Point::new(acc, (max_length - self.axis.invert().get_axis(s)) / 2.0)
                }
                Axis::Vertical => {
                    Point::new((max_length - self.axis.invert().get_axis(s)) / 2.0, acc)
                }
            };
            acc += self.axis.get_axis(s) + self.spacing as f64;
            handle.position_widget(Rect::from_origin_size(pos, s), id)
        }

        match self.axis {
            Axis::Horizontal => Size::new(
                Itertools::intersperse(layouts.iter().map(|(s, _)| s.width), self.spacing as f64)
                    .sum(),
                max_length,
            ),
            Axis::Vertical => Size::new(
                max_length,
                Itertools::intersperse(layouts.iter().map(|(s, _)| s.height), self.spacing as f64)
                    .sum(),
            ),
        }
    }

    fn event(&mut self, event: WidgetEvent, handle: &mut EventHandle<C>) {
        handle.propagate_event(event, self.children.iter_mut())
    }
}

#[derive(Deserialize, WidgetBuilder, Debug, Clone)]
#[serde(deny_unknown_fields)]
#[widget(
    name = "HStack",
    type_path = "::gui::gui_widget::HVStack<#child>",
    init_path = "new_horizontal"
)]
pub struct HStackBuilder {
    #[widget(property = "set_spacing", default = 0_10f32)]
    spacing: Option<Var<f32>>,
    #[widget(children = "widgets")]
    children: Option<Vec<WidgetDeclaration>>,
}

#[derive(Deserialize, WidgetBuilder, Debug, Clone)]
#[serde(deny_unknown_fields)]
#[widget(
    name = "VStack",
    type_path = "::gui::gui_widget::HVStack<#child>",
    init_path = "new_vertical"
)]
pub struct VStackBuilder {
    #[widget(property = "set_spacing", default = 0_10f32)]
    spacing: Option<Var<f32>>,
    #[widget(children = "widgets")]
    children: Option<Vec<WidgetDeclaration>>,
}
