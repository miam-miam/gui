use crate::positions::WidgetInfo;
use crate::widget::{RuntimeID, Widget, WidgetEvent, WidgetID};
use crate::{LayoutConstraints, MultiComponent, Point, Size, ToComponent};
use glazier::kurbo::{Affine, Rect};
use glazier::{Cursor, WindowHandle};
use parley::FontContext;
use vello::{SceneBuilder, SceneFragment};

#[derive(Clone)]
pub struct Handle {
    pub fcx: FontContext,
    pub window: WindowHandle,
    pub info: WidgetInfo,
}

impl Default for Handle {
    fn default() -> Self {
        Self {
            fcx: FontContext::new(),
            window: WindowHandle::default(),
            info: WidgetInfo::default(),
        }
    }
}

impl Handle {
    pub fn if_window<F: FnOnce(&mut WindowHandle)>(&mut self, f: F) {
        if self.window != WindowHandle::default() {
            f(&mut self.window);
        }
    }
}

pub struct UpdateHandle<'a> {
    handle: &'a mut Handle,
    runtime_id: RuntimeID,
    resize: bool,
}

impl<'a> UpdateHandle<'a> {
    pub fn new(handle: &'a mut Handle, runtime_id: RuntimeID) -> Self {
        Self {
            handle,
            runtime_id,
            resize: false,
        }
    }
    pub fn get_fcx(&mut self) -> &mut FontContext {
        &mut self.handle.fcx
    }
    pub fn resize(&mut self) {
        self.resize = true;
    }
    pub fn unwrap(self) -> bool {
        self.resize
    }
    pub fn invalidate_id(&mut self, id: WidgetID) {
        let rect = self.handle.info.get_rect(self.runtime_id, id);
        self.handle.if_window(|w| w.invalidate_rect(rect));
    }
    pub fn invalidate_rect(&mut self, id: WidgetID, local_rect: Rect) {
        let global_rect = self.handle.info.get_rect(self.runtime_id, id);
        self.handle
            .window
            .invalidate_rect(local_rect + global_rect.origin().to_vec2())
    }
}

pub struct RenderHandle<'a, T: ToComponent> {
    handle: &'a mut Handle,
    runtime_id: RuntimeID,
    resize: bool,
    comp_struct: &'a mut T,
    held_components: &'a mut T::HeldComponents,
}

impl<'a, T: ToComponent> RenderHandle<'a, T> {
    pub fn new(
        handle: &'a mut Handle,
        runtime_id: RuntimeID,
        comp_struct: &'a mut T,
        held_components: &'a mut T::HeldComponents,
    ) -> Self {
        Self {
            handle,
            runtime_id,
            resize: false,
            comp_struct,
            held_components,
        }
    }

    pub fn get_fcx(&mut self) -> &mut FontContext {
        &mut self.handle.fcx
    }

    pub fn resize(&mut self) {
        self.resize = true;
    }

    pub fn unwrap(self) -> bool {
        self.resize
    }

    pub fn render_widgets<'b, W: Widget<T> + 'b>(
        &mut self,
        scene: &mut SceneBuilder,
        iter: impl Iterator<Item = &'b mut W>,
    ) {
        let mut parent_origin: Option<Point> = None;
        for w in iter {
            let id = w.id();
            let child_pos = self.handle.info.get_rect(self.runtime_id, id);
            let parent_origin = *parent_origin.get_or_insert_with(|| {
                self.comp_struct
                    .get_parent(self.runtime_id, id)
                    .map_or_else(Point::default, |(parent_runtime, parent_widget)| {
                        self.handle
                            .info
                            .get_rect(parent_runtime, parent_widget)
                            .origin()
                    })
            });
            let vector = child_pos.origin() - parent_origin;

            let mut fragment = SceneFragment::new();
            let mut builder = SceneBuilder::for_fragment(&mut fragment);
            w.render(&mut builder, self);

            scene.append(&fragment, Some(Affine::translate(vector)));
        }
    }

    pub fn render_component(&mut self, scene: &mut SceneBuilder, component_id: WidgetID) {
        self.held_components
            .render(component_id, scene, self.handle);
    }

    pub fn is_active(&self, id: WidgetID) -> bool {
        self.handle.info.is_active(self.runtime_id, id)
    }

    pub fn is_hovered(&self, id: WidgetID) -> bool {
        self.handle.info.is_hovered(self.runtime_id, id)
    }

    pub fn get_global_rect(&self, id: WidgetID) -> Rect {
        self.handle.info.get_rect(self.runtime_id, id)
    }

    pub fn get_local_rect(&self, id: WidgetID) -> Rect {
        let global = self.get_global_rect(id);
        global.with_origin((0.0, 0.0))
    }
}

pub struct ResizeHandle<'a, T: ToComponent> {
    handle: &'a mut Handle,
    runtime_id: RuntimeID,
    comp_struct: &'a mut T,
    held_components: &'a mut T::HeldComponents,
}

impl<'a, T: ToComponent> ResizeHandle<'a, T> {
    pub fn new(
        handle: &'a mut Handle,
        runtime_id: RuntimeID,
        comp_struct: &'a mut T,
        held_components: &'a mut T::HeldComponents,
    ) -> Self {
        Self {
            handle,
            comp_struct,
            runtime_id,
            held_components,
        }
    }
    pub fn get_fcx(&mut self) -> &mut FontContext {
        &mut self.handle.fcx
    }

    pub fn position_widget(&mut self, rect: Rect, child_id: WidgetID) {
        self.handle
            .info
            .position_widget(self.runtime_id, child_id, rect);
    }

    pub fn layout_widget<W: Widget<T>>(
        &mut self,
        origin: Point,
        child: &mut W,
        constraints: LayoutConstraints,
    ) -> Size {
        let s = child.resize(constraints, self);
        self.position_widget(Rect::from_origin_size(origin, s), child.id());
        s
    }

    pub fn layout_component(
        &mut self,
        component_id: WidgetID,
        constraints: LayoutConstraints,
    ) -> Size {
        let s = self
            .held_components
            .resize(component_id, constraints, self.handle);
        s
    }

    pub fn get_handler(&mut self) -> &mut T {
        self.comp_struct
    }
}

pub struct EventHandle<'a, T: ToComponent> {
    handle: &'a mut Handle,
    runtime_id: RuntimeID,
    resize: bool,
    // TODO fix to also use Runtime IDs
    events_to_propagate: Vec<(WidgetID, WidgetEvent<'static>)>,
    comp_struct: &'a mut T,
    held_components: &'a mut T::HeldComponents,
}

impl<'a, T: ToComponent> EventHandle<'a, T> {
    pub fn new(
        handle: &'a mut Handle,
        runtime_id: RuntimeID,
        comp_struct: &'a mut T,
        held_components: &'a mut T::HeldComponents,
    ) -> Self {
        Self {
            handle,
            resize: false,
            runtime_id,
            events_to_propagate: vec![],
            comp_struct,
            held_components,
        }
    }
    pub fn get_fcx(&mut self) -> &mut FontContext {
        &mut self.handle.fcx
    }

    pub fn invalidate_id(&mut self, id: WidgetID) {
        let rect = self.handle.info.get_rect(self.runtime_id, id);
        self.handle.if_window(|w| w.invalidate_rect(rect));
    }

    pub fn invalidate_rect(&mut self, id: WidgetID, local_rect: Rect) {
        let global_rect = self.handle.info.get_rect(self.runtime_id, id);
        self.handle
            .window
            .invalidate_rect(local_rect + global_rect.origin().to_vec2())
    }

    pub fn resize(&mut self) {
        self.resize = true;
    }

    pub fn get_local_point(&self, id: WidgetID, pos: Point) -> Point {
        let global_rect = self.handle.info.get_rect(self.runtime_id, id);
        (pos - global_rect.origin()).to_point()
    }

    pub fn get_global_rect(&self, id: WidgetID) -> Rect {
        self.handle.info.get_rect(self.runtime_id, id)
    }

    pub fn get_local_rect(&self, id: WidgetID) -> Rect {
        let global = self.get_global_rect(id);
        global.with_origin((0.0, 0.0))
    }

    pub fn propagate_event<'b, W: Widget<T> + 'b>(
        &mut self,
        event: WidgetEvent,
        iter: impl Iterator<Item = &'b mut W>,
    ) {
        for w in iter {
            let id = w.id();

            if let Some(point) = event.get_point() {
                let child_pos = self.handle.info.get_rect(self.runtime_id, id);
                if child_pos.contains(point) {
                    w.event(event, self);
                    return;
                }
            }

            w.event(event.clone(), self);
        }
    }

    pub fn propagate_component_event(&mut self, component_id: WidgetID, event: WidgetEvent) {
        self.held_components
            .propagate_event(component_id, event, self.handle);
    }

    pub fn unwrap(self) -> (bool, Vec<(WidgetID, WidgetEvent<'static>)>) {
        (self.resize, self.events_to_propagate)
    }

    pub fn set_active(&mut self, id: WidgetID, active: bool) {
        if let Some(old_id) = self.handle.info.get_active_widget() {
            if !active && old_id != (self.runtime_id, id) {
                return;
            }
            if !(active && old_id == (self.runtime_id, id)) {
                self.events_to_propagate
                    .push((old_id.1, WidgetEvent::ActiveChange));
            }
        }
        *self.handle.info.set_active_widget() = active.then_some((self.runtime_id, id));
    }

    pub fn add_hover(&mut self, id: WidgetID) -> bool {
        self.handle.info.add_hover(self.runtime_id, id)
    }

    pub fn is_active(&self, id: WidgetID) -> bool {
        self.handle.info.is_active(self.runtime_id, id)
    }

    pub fn is_hovered(&self, id: WidgetID) -> bool {
        self.handle.info.is_hovered(self.runtime_id, id)
    }
    pub fn set_cursor(&mut self, cursor: &Cursor) {
        self.handle.if_window(|w| w.set_cursor(cursor))
    }
    pub fn get_handler(&mut self) -> &mut T {
        self.comp_struct
    }
}
