use proc_macro2::{Ident, TokenStream};
use quote::quote;
use serde::Deserialize;

use gui_core::glazier::kurbo::Size;
use gui_core::layout::LayoutConstraints;
use gui_core::parse::var::ComponentVar;
use gui_core::widget::{
    EventHandle, RenderHandle, ResizeHandle, RuntimeID, UpdateHandle, Widget, WidgetBuilder,
    WidgetEvent, WidgetID,
};
use gui_core::{Point, SceneBuilder, ToComponent};
use gui_derive::WidgetBuilder;

pub struct CompHolder {
    id: WidgetID,
    child_id: Option<RuntimeID>,
}

impl CompHolder {
    pub fn new(id: WidgetID) -> Self {
        CompHolder { id, child_id: None }
    }

    pub fn set_child(&mut self, id: RuntimeID, handle: &mut UpdateHandle) {
        self.child_id = Some(id);
        handle.resize();
    }
}

impl<C: ToComponent> Widget<C> for CompHolder {
    fn id(&self) -> WidgetID {
        self.id
    }

    fn render(&mut self, scene: &mut SceneBuilder, handle: &mut RenderHandle<C>) {
        if let Some(child_id) = self.child_id {
            handle.render_component(scene, child_id);
        }
    }

    fn resize(&mut self, constraints: LayoutConstraints, handle: &mut ResizeHandle<C>) -> Size {
        match self.child_id {
            Some(child_id) => handle.layout_component(Point::ZERO, child_id, constraints),
            None => Size::default(),
        }
    }

    fn event(&mut self, event: WidgetEvent, handle: &mut EventHandle<C>) {
        if let Some(child_id) = self.child_id {
            handle.propagate_component_event(child_id, event);
        }
    }
}

#[derive(Deserialize, WidgetBuilder, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
#[widget(
    name = "CompHolder",
    type_path = "::gui::gui_widget::CompHolder",
    init_path = "new"
)]
pub struct CompHolderBuilder {
    #[widget(component = "set_child")]
    pub component: Option<ComponentVar>,
}
