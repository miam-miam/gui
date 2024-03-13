pub mod common;
pub mod parse;
pub mod widget;

mod comp_holder;
mod handles;
pub mod layout;
mod positions;

pub use glazier;
pub use glazier::kurbo::Point;
pub use layout::LayoutConstraints;
pub use parley;
pub use parse::colour::Colour;
pub use parse::var::Var;
use std::any::Any;
pub use vello;
pub use vello::kurbo::Size;

pub use crate::comp_holder::CompHolder;
use crate::handles::Handle;
use crate::widget::{RuntimeID, WidgetEvent, WidgetID};
pub use parley::font::FontContext;
pub use vello::SceneBuilder;

#[allow(dead_code)]
struct TestBoxable {
    test: Box<dyn Component>,
}

pub trait Component {
    fn render(&mut self, scene: &mut SceneBuilder, handle: &mut Handle) -> bool;
    fn update_vars(&mut self, force_update: bool, handle: &mut Handle) -> bool;
    fn resize(&mut self, constraints: LayoutConstraints, handle: &mut Handle) -> Size;
    fn propagate_event(&mut self, event: WidgetEvent, handle: &mut Handle) -> bool;
    fn get_parent(
        &self,
        runtime_id: RuntimeID,
        widget_id: WidgetID,
    ) -> Option<(RuntimeID, WidgetID)>;
    fn get_id(&self, name: &str) -> Option<(RuntimeID, WidgetID)>;
    // Only used for the test harness.
    fn get_comp_struct(&mut self) -> &mut dyn Any;
    fn event(
        &mut self,
        runtime_id: RuntimeID,
        widget_id: WidgetID,
        event: WidgetEvent,
        handle: &mut Handle,
    ) -> bool;
    fn id(&self) -> RuntimeID;
}

pub trait MultiComponent {
    fn render(
        &mut self,
        runtime_id: RuntimeID,
        scene: &mut SceneBuilder,
        handle: &mut Handle,
    ) -> bool;
    fn update_vars(
        &mut self,
        runtime_id: RuntimeID,
        force_update: bool,
        handle: &mut Handle,
    ) -> bool;
    fn force_update_vars(&mut self, handle: &mut Handle) -> bool;
    fn resize(
        &mut self,
        runtime_id: RuntimeID,
        constraints: LayoutConstraints,
        handle: &mut Handle,
    ) -> Size;

    fn propagate_event(
        &mut self,
        runtime_id: RuntimeID,
        event: WidgetEvent,
        handle: &mut Handle,
    ) -> bool;
    fn event(
        &mut self,
        runtime_id: RuntimeID,
        widget_id: WidgetID,
        event: WidgetEvent,
        handle: &mut Handle,
    ) -> bool;
    fn get_parent(
        &self,
        runtime_id: RuntimeID,
        widget_id: WidgetID,
    ) -> Option<(RuntimeID, WidgetID)>;
    fn get_id(&self, name: &str) -> Option<(RuntimeID, WidgetID)>;
}

pub trait ToComponent {
    type Component: Component;
    type HeldComponents: MultiComponent;
    fn to_component_holder(self, runtime_id: RuntimeID) -> Self::Component;
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
