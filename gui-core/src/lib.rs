pub mod common;
pub mod parse;
pub mod widget;

mod comp_holder;
mod handles;
pub mod layout;

pub use glazier;
pub use glazier::kurbo::Point;
use glazier::kurbo::Rect;
pub use layout::LayoutConstraints;
pub use parley;
pub use parse::colour::Colour;
pub use parse::var::Var;
use std::any::Any;
pub use vello;
pub use vello::kurbo::Size;

pub use crate::comp_holder::CompHolder;
use crate::handles::Handle;
use crate::widget::{WidgetEvent, WidgetID};
pub use parley::font::FontContext;
pub use vello::SceneBuilder;

#[allow(dead_code)]
struct TestBoxable {
    test: Box<dyn Component>,
}

pub trait Component {
    fn render(
        &mut self,
        scene: &mut SceneBuilder,
        handle: &mut Handle,
        global_positions: &mut [Rect],
        active_widget: &mut Option<WidgetID>,
        hovered_widgets: &[WidgetID],
    ) -> bool;
    fn update_vars(
        &mut self,
        force_update: bool,
        handle: &mut Handle,
        global_positions: &[Rect],
    ) -> bool;
    fn resize(
        &mut self,
        constraints: LayoutConstraints,
        handle: &mut Handle,
        local_positions: &mut [Rect],
    ) -> Size;

    fn propagate_event(
        &mut self,
        event: WidgetEvent,
        handle: &mut Handle,
        global_positions: &[Rect],
        active_widget: &mut Option<WidgetID>,
        hovered_widgets: &mut Vec<WidgetID>,
    ) -> bool;
    fn largest_id(&self) -> WidgetID;
    fn get_parent(&self, id: WidgetID) -> Option<WidgetID>;
    fn get_id(&self, name: &str) -> Option<WidgetID>;
    // Only used for the test harness.
    fn get_comp_struct(&mut self) -> &mut dyn Any;
    fn event(
        &mut self,
        id: WidgetID,
        event: WidgetEvent,
        handle: &mut Handle,
        global_positions: &[Rect],
        active_widget: &mut Option<WidgetID>,
        hovered_widgets: &mut Vec<WidgetID>,
    ) -> bool;
}

pub trait MultiComponent {
    fn render(
        &mut self,
        comp_id: WidgetID,
        scene: &mut SceneBuilder,
        handle: &mut Handle,
        global_positions: &mut [Rect],
        active_widget: &mut Option<WidgetID>,
        hovered_widgets: &[WidgetID],
    ) -> bool;
    fn update_vars(
        &mut self,
        comp_id: WidgetID,
        force_update: bool,
        handle: &mut Handle,
        global_positions: &[Rect],
    ) -> bool;
    fn resize(
        &mut self,
        comp_id: WidgetID,
        constraints: LayoutConstraints,
        handle: &mut Handle,
        local_positions: &mut [Rect],
    ) -> Size;

    fn propagate_event(
        &mut self,
        comp_id: WidgetID,
        event: WidgetEvent,
        handle: &mut Handle,
        global_positions: &[Rect],
        active_widget: &mut Option<WidgetID>,
        hovered_widgets: &mut Vec<WidgetID>,
    ) -> bool;
    fn event(
        &mut self,
        comp_id: WidgetID,
        id: WidgetID,
        event: WidgetEvent,
        handle: &mut Handle,
        global_positions: &[Rect],
        active_widget: &mut Option<WidgetID>,
        hovered_widgets: &mut Vec<WidgetID>,
    ) -> bool;
}

pub trait ToComponent {
    type Component: Component;
    type HeldComponents: MultiComponent;
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

pub trait ComponentTypeInfo {
    type ToComponent: ToComponent;
}

pub trait ComponentHolder<T: Variable>
where
    T::VarType: ToComponent,
{
    fn comp_holder(&mut self) -> &mut CompHolder<T::VarType>;
}
