use std::any::Any;

use gui_core::{Component, LayoutConstraints, MultiComponent, SceneBuilder, Size, ToComponent, ToHandler};
use gui_core::widget::{Handle, RuntimeID, WidgetEvent, WidgetID};

pub struct Handler;

impl ToHandler for Handler {
    type BaseHandler = ToComp;
}

pub struct ToComp;

impl ToComponent for ToComp {
    type Component = Comp;
    type HeldComponents = MultiComp;

    fn to_component_holder(self, _runtime_id: RuntimeID) -> Self::Component {
        unimplemented!()
    }

    fn get_parent(&self, _id: WidgetID) -> Option<WidgetID> {
        unimplemented!()
    }

    fn get_id(&self, _name: &str) -> Option<WidgetID> {
        unimplemented!()
    }
}

pub struct Comp;

impl Component for Comp {
    fn render(&mut self, _scene: &mut SceneBuilder, _handle: &mut Handle) -> bool {
        unimplemented!()
    }

    fn update_vars(&mut self, _force_update: bool, _handle: &mut Handle) -> bool {
        unimplemented!()
    }

    fn resize(&mut self, _constraints: LayoutConstraints, _handle: &mut Handle) -> Size {
        unimplemented!()
    }

    fn propagate_event(&mut self, _event: WidgetEvent, _handle: &mut Handle) -> bool {
        unimplemented!()
    }

    fn get_parent(&self, _runtime_id: RuntimeID, _widget_id: WidgetID) -> Option<(RuntimeID, WidgetID)> {
        unimplemented!()
    }

    fn get_id(&self, _name: &str) -> Option<(RuntimeID, WidgetID)> {
        unimplemented!()
    }

    fn get_comp_struct(&mut self) -> &mut dyn Any {
        unimplemented!()
    }

    fn event(&mut self, _runtime_id: RuntimeID, _widget_id: WidgetID, _event: WidgetEvent, _handle: &mut Handle) -> bool {
        unimplemented!()
    }

    fn id(&self) -> RuntimeID {
        unimplemented!()
    }
}

pub struct MultiComp;

impl MultiComponent for MultiComp {
    fn render(
        &mut self,
        _runtime_id: RuntimeID,
        _scene: &mut SceneBuilder,
        _handle: &mut Handle,
    ) -> bool {
        unimplemented!()
    }
    fn update_vars(
        &mut self,
        _runtime_id: RuntimeID,
        _force_update: bool,
        _handle: &mut Handle,
    ) -> bool {
        unimplemented!()
    }
    fn update_all_vars(&mut self, _force_update: bool, _handle: &mut Handle) -> bool {
        unimplemented!()
    }
    fn resize(
        &mut self,
        _runtime_id: RuntimeID,
        _constraints: LayoutConstraints,
        _handle: &mut Handle,
    ) -> Size {
        unimplemented!()
    }
    fn propagate_event(
        &mut self,
        _runtime_id: RuntimeID,
        _event: WidgetEvent,
        _handle: &mut Handle,
    ) -> bool {
        unimplemented!()
    }
    fn event(
        &mut self,
        _runtime_id: RuntimeID,
        _widget_id: WidgetID,
        _event: WidgetEvent,
        _handle: &mut Handle,
    ) -> bool {
        unimplemented!()
    }
    fn get_parent(
        &self,
        _runtime_id: RuntimeID,
        _widget_id: WidgetID,
    ) -> Option<(RuntimeID, WidgetID)> {
        unimplemented!()
    }
    fn get_id(&self, _name: &str) -> Option<(RuntimeID, WidgetID)> {
        unimplemented!()
    }
}