pub mod common;
pub mod parse;
pub mod widget;

mod handles;
pub mod layout;

pub use layout::LayoutConstraints;
pub use parse::colour::Colour;
pub use parse::var::Var;
use std::any::Any;
pub use vello::kurbo::Size;

pub use glazier;
pub use glazier::kurbo::Point;
use glazier::kurbo::Rect;
pub use parley;
pub use vello;

use crate::handles::Handle;
use crate::widget::{WidgetEvent, WidgetID};
pub use parley::font::FontContext;
pub use vello::SceneBuilder;

#[allow(dead_code)]
struct TestBoxable {
    test: Box<dyn Component>,
}

pub trait Component {
    fn render<'a>(
        &mut self,
        scene: SceneBuilder,
        handle: &'a mut Handle,
        global_positions: &'a mut [Rect],
        active_widget: &'a mut Option<WidgetID>,
        hovered_widgets: &'a [WidgetID],
    ) -> bool;
    fn update_vars<'a>(
        &mut self,
        force_update: bool,
        handle: &'a mut Handle,
        global_positions: &'a [Rect],
    ) -> bool;
    fn resize<'a>(
        &mut self,
        constraints: LayoutConstraints,
        handle: &'a mut Handle,
        local_positions: &'a mut [Rect],
    ) -> Size;

    fn propagate_event<'a>(
        &mut self,
        event: WidgetEvent,
        handle: &'a mut Handle,
        global_positions: &'a [Rect],
        active_widget: &'a mut Option<WidgetID>,
        hovered_widgets: &'a mut Vec<WidgetID>,
    ) -> bool;
    fn largest_id(&self) -> WidgetID;
    fn get_parent(&self, id: WidgetID) -> Option<WidgetID>;
    fn get_id(&self, name: &str) -> Option<WidgetID>;
    // Only used for the test harness.
    fn get_comp_struct(&mut self) -> &mut dyn Any;
    fn event<'a>(
        &mut self,
        id: WidgetID,
        event: WidgetEvent,
        handle: &'a mut Handle,
        global_positions: &'a [Rect],
        active_widget: &'a mut Option<WidgetID>,
        hovered_widgets: &'a mut Vec<WidgetID>,
    ) -> bool;
}

pub trait ToComponent {
    type Component: Component;
    fn to_component_holder(self) -> Self::Component;
    fn largest_id(&self) -> WidgetID;
    fn get_parent(&self, id: WidgetID) -> Option<WidgetID>;
    fn get_id(&self, name: &str) -> Option<WidgetID>;
}

pub trait Variable {
    type VarType;
}

pub trait ToHandler {
    type BaseHandler;
}

pub trait Update<T: Variable> {
    fn is_updated(&self) -> bool;
    fn value(&self) -> T::VarType;
    fn reset(&mut self) {}
}
