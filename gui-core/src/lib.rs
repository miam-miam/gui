use std::any::Any;

pub use glazier;
pub use glazier::kurbo::Point;
pub use parley;
pub use parley::font::FontContext;
pub use vello;
pub use vello::kurbo::Size;
pub use vello::SceneBuilder;

pub use layout::LayoutConstraints;
pub use parse::colour::Colour;
pub use parse::var::Var;
pub use single_or_multi::{Children, MutWidgetChildren, WidgetChildren};

pub use crate::comp_holder::CompHolder;
use crate::handles::Handle;
use crate::widget::{RuntimeID, WidgetEvent, WidgetID};

pub mod common;
pub mod parse;
pub mod widget;

mod comp_holder;
mod handles;
pub mod layout;
mod positions;
mod single_or_multi;

#[allow(dead_code)]
struct TestBoxable {
    test: Box<dyn Component>,
}

pub trait Component {
    /// Function called to render a component to a `scene`
    fn render(&mut self, scene: &mut SceneBuilder, handle: &mut Handle) -> bool;
    /// Called before every render or resize, `force_update` will update variables even if they
    /// have not changed. It will usually be set when a component is rendered for the first time.
    fn update_vars(&mut self, force_update: bool, handle: &mut Handle) -> bool;
    /// Resize the component based on a set of [`LayoutConstraints`] constraints. The returned [`Size`] must be observed.
    fn resize(&mut self, constraints: LayoutConstraints, handle: &mut Handle) -> Size;
    /// Propagates a [`WidgetEvent`] through the widget tree.
    fn propagate_event(&mut self, event: WidgetEvent, handle: &mut Handle) -> bool;
    /// Get the parent of a widget. Return `None` if the widget has no parent.
    fn get_parent(
        &self,
        runtime_id: RuntimeID,
        widget_id: WidgetID,
    ) -> Option<(RuntimeID, WidgetID)>;
    /// Return the first matching widget with the given name.
    fn get_id(&self, name: &str) -> Option<(RuntimeID, WidgetID)>;
    // Only used for the test harness.
    fn get_comp_struct(&mut self) -> &mut dyn Any;
    /// Send a [`WidgetEvent`] to a specific widget.
    fn event(
        &mut self,
        runtime_id: RuntimeID,
        widget_id: WidgetID,
        event: WidgetEvent,
        handle: &mut Handle,
    ) -> bool;
    /// Get the component's ID.
    fn id(&self) -> RuntimeID;
}

/// Similar trait to [`Component`] that allows a specific component to be selected using a [`RuntimeID`].
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
    /// Same as `update_vars` but updates all components.
    fn update_all_vars(&mut self, force_update: bool, handle: &mut Handle) -> bool;
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

/// Trait that is derived on user-defined components using the derive macro.
pub trait ToComponent {
    /// The actual generated Component
    type Component: Component;
    /// The type of components used by the generated Component
    type HeldComponents: MultiComponent;
    fn to_component_holder(self, runtime_id: RuntimeID) -> Self::Component;
    fn get_parent(&self, id: WidgetID) -> Option<WidgetID>;
    fn get_id(&self, name: &str) -> Option<WidgetID>;
}

/// Trait used to respond to messages from other components
pub trait OnMessage {
    type Message;
    fn on_message(&mut self, message: Self::Message);
}

/// Trait implemented by generated variables
pub trait Variable {
    type VarType;
}

/// Trait implemented by generated handler names
pub trait ToHandler {
    type BaseHandler: ToComponent;
}

/// Trait that broadcast the value of a variable back to the component.
pub trait Update<T: Variable> {
    /// Decides when a variable should be re-evaluated and `value` run.
    fn is_updated(&self) -> bool;
    /// The current value of the variable
    fn value(&self) -> T::VarType;
    /// Only used by `Updateable`, resets updated so that it is not marked as updated on the next frame.
    fn reset(&mut self) {}
}

/// Trait used to gather information about the type of different user-defined components using the `type_registry` macro.
pub trait ComponentTypeInfo {
    type ToComponent: ToComponent;
}

/// Trait used to get the stored component for a given variable. This trait should not need to be implemented manually.
pub trait ComponentHolder<T: Variable>
where
    T::VarType: ToComponent + OnMessage,
{
    fn comp_holder(&mut self) -> &mut CompHolder<T::VarType>;
}
