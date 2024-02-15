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
use glazier::{PointerEvent, WindowHandle};
pub use parley;
pub use vello;

use crate::widget::{WidgetEvent, WidgetID};
pub use parley::font::FontContext;
pub use vello::SceneBuilder;

#[allow(dead_code)]
struct TestBoxable {
    test: Box<dyn Component<Handler = ()>>,
}

pub trait Component {
    type Handler;
    fn render(&mut self, scene: SceneBuilder, fcx: &mut FontContext);
    fn update_vars(&mut self, force_update: bool);
    fn resize(&mut self, constraints: LayoutConstraints, fcx: &mut FontContext) -> Size;
    fn get_parent(&self, id: WidgetID) -> Option<WidgetID>;
    fn event(&mut self, id: WidgetID, event: WidgetEvent);
    fn get_handler(&mut self) -> &mut Self::Handler;
    fn pointer_down(&mut self, local_pos: Point, event: &PointerEvent, window: &WindowHandle);
    fn pointer_up(&mut self, local_pos: Point, event: &PointerEvent, window: &WindowHandle);
    fn pointer_move(&mut self, local_pos: Point, event: &PointerEvent, window: &WindowHandle);
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
