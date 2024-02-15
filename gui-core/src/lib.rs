pub mod common;
pub mod parse;
pub mod widget;

mod handles;
pub mod layout;

pub use layout::LayoutConstraints;
pub use parse::colour::Colour;
pub use parse::var::Var;
pub use vello::kurbo::Size;

pub use glazier;
pub use glazier::kurbo::Point;
use glazier::kurbo::Rect;
use glazier::{PointerEvent, WindowHandle};
pub use parley;
pub use vello;

use crate::handles::{EventHandle, Handle};
use crate::widget::{WidgetEvent, WidgetID};
pub use parley::font::FontContext;
pub use vello::SceneBuilder;

#[allow(dead_code)]
struct TestBoxable {
    test: Box<dyn Component<Handler = ()>>,
}

pub trait Component {
    type Handler;
    fn render<'a>(
        &mut self,
        scene: SceneBuilder,
        handle: &'a mut Handle,
        global_positions: &'a mut [Rect],
        active_widget: Option<WidgetID>,
        hovered_widgets: &'a [WidgetID],
    ) -> (bool, Option<WidgetID>);
    fn update_vars(&mut self, force_update: bool);
    fn resize<'a>(
        &mut self,
        constraints: LayoutConstraints,
        handle: &'a mut Handle,
        local_positions: &'a mut [Rect],
    ) -> Size;

    fn propagate_event<'a>(
        &mut self,
        event: WidgetEvent,
        global_positions: &'a [Rect],
        hovered_widgets: &'a mut Vec<WidgetID>,
    ) -> (bool, Option<WidgetID>);
    fn largest_id(&self) -> WidgetID;
    fn get_parent(&self, id: WidgetID) -> Option<WidgetID>;
    fn event<'a>(
        &mut self,
        id: WidgetID,
        event: WidgetEvent,
        global_positions: &'a [Rect],
        active_widget: Option<WidgetID>,
        hovered_widgets: &'a mut Vec<WidgetID>,
    ) -> (bool, Option<WidgetID>);
    fn get_handler(&mut self) -> &mut Self::Handler;
}

pub trait ToComponent {
    type Component: Component;
    fn to_component_holder(self) -> Self::Component;
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

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
