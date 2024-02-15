use crate::widget::{Widget, WidgetEvent, WidgetID};
use crate::{Component, LayoutConstraints, Point, Size};
use glazier::kurbo::{Affine, Rect};
use glazier::WindowHandle;
use parley::FontContext;
use vello::{SceneBuilder, SceneFragment};

#[derive(Clone)]
pub struct Handle {
    pub fcx: FontContext,
    pub window: WindowHandle,
}

impl Default for Handle {
    fn default() -> Self {
        Self {
            fcx: FontContext::new(),
            window: WindowHandle::default(),
        }
    }
}

pub struct RenderHandle<'a, C> {
    handle: &'a mut Handle,
    global_positions: &'a mut [Rect],
    resize: bool,
    active_widget: &'a mut Option<WidgetID>,
    hovered_widgets: &'a [WidgetID],
    component: &'a mut C,
}

impl<'a, C: Component> RenderHandle<'a, C> {
    pub fn new(
        handle: &'a mut Handle,
        global_positions: &'a mut [Rect],
        active_widget: &'a mut Option<WidgetID>,
        hovered_widgets: &'a [WidgetID],
        component: &'a mut C,
    ) -> Self {
        Self {
            handle,
            global_positions,
            active_widget,
            hovered_widgets,
            resize: false,
            component,
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

    pub fn render_widgets<'b, W: Widget<C> + 'b>(
        &mut self,
        scene: &mut SceneBuilder,
        iter: impl Iterator<Item = &'b mut W>,
    ) {
        let mut parent_origin: Option<Point> = None;
        for w in iter {
            let id = w.id();
            let child_pos = self.global_positions[id.widget_id() as usize];
            let parent_origin = *parent_origin.get_or_insert_with(|| {
                self.component
                    .get_parent(id)
                    .map_or_else(Point::default, |parent_id| {
                        self.global_positions[parent_id.widget_id() as usize].origin()
                    })
            });
            let vector = child_pos.origin() - parent_origin;

            let mut fragment = SceneFragment::new();
            let mut builder = SceneBuilder::for_fragment(&mut fragment);
            w.render(&mut builder, self);

            scene.append(&fragment, Some(Affine::translate(vector)));
        }
    }

    pub fn is_active(&self, id: WidgetID) -> bool {
        self.active_widget == &Some(id)
    }

    pub fn is_hovered(&self, id: WidgetID) -> bool {
        self.hovered_widgets.contains(&id)
    }

    pub fn get_global_rect(&self, id: WidgetID) -> Rect {
        self.global_positions[id.widget_id() as usize]
    }

    pub fn get_local_rect(&self, id: WidgetID) -> Rect {
        let global = self.get_global_rect(id);
        let parent_pos = self
            .component
            .get_parent(id)
            .map_or_else(Point::default, |id| {
                self.global_positions[id.widget_id() as usize].origin()
            });
        let vector = global.origin() - parent_pos;
        global.with_origin(vector.to_point())
    }
}

pub struct ResizeHandle<'a, C> {
    handle: &'a mut Handle,
    local_positions: &'a mut [Rect],
    _component: &'a mut C,
}

impl<'a, C: Component> ResizeHandle<'a, C> {
    pub fn new(
        handle: &'a mut Handle,
        local_positions: &'a mut [Rect],
        component: &'a mut C,
    ) -> Self {
        Self {
            handle,
            _component: component,
            local_positions,
        }
    }
    pub fn get_fcx(&mut self) -> &mut FontContext {
        &mut self.handle.fcx
    }

    pub fn position_widget(&mut self, rect: Rect, child_id: WidgetID) {
        self.local_positions[child_id.widget_id() as usize] = rect;
    }

    pub fn layout_widget<W: Widget<C>>(
        &mut self,
        origin: Point,
        child: &mut W,
        constraints: LayoutConstraints,
    ) -> Size {
        let s = child.resize(constraints, self);
        self.position_widget(Rect::from_origin_size(origin, s), child.id());
        s
    }
}

pub struct EventHandle<'a, C> {
    handle: &'a mut Handle,
    global_positions: &'a [Rect],
    resize: bool,
    active_widget: &'a mut Option<WidgetID>,
    hovered_widgets: &'a mut Vec<WidgetID>,
    component: &'a mut C,
}

impl<'a, C: Component> EventHandle<'a, C> {
    pub fn new(
        handle: &'a mut Handle,
        global_positions: &'a [Rect],
        active_widget: &'a mut Option<WidgetID>,
        hovered_widgets: &'a mut Vec<WidgetID>,
        component: &'a mut C,
    ) -> Self {
        Self {
            handle,
            resize: false,
            global_positions,
            active_widget,
            hovered_widgets,
            component,
        }
    }
    pub fn get_fcx(&mut self) -> &mut FontContext {
        &mut self.handle.fcx
    }

    pub fn invalidate_id(&mut self, id: WidgetID) {
        let rect = self.global_positions[id.widget_id() as usize];
        self.handle.window.invalidate_rect(rect);
    }

    pub fn invalidate_rect(&mut self, id: WidgetID, local_rect: Rect) {
        let global_rect = self.global_positions[id.widget_id() as usize];
        self.handle
            .window
            .invalidate_rect(local_rect + global_rect.origin().to_vec2())
    }

    pub fn resize(&mut self) {
        self.resize = true;
    }

    pub fn get_local_point(&self, id: WidgetID, pos: Point) -> Point {
        let global_rect = self.global_positions[id.widget_id() as usize];
        (pos - global_rect.origin()).to_point()
    }

    pub fn get_global_rect(&self, id: WidgetID) -> Rect {
        self.global_positions[id.widget_id() as usize]
    }

    pub fn get_local_rect(&self, id: WidgetID) -> Rect {
        let global = self.get_global_rect(id);
        let parent_pos = self
            .component
            .get_parent(id)
            .map_or_else(Point::default, |id| {
                self.global_positions[id.widget_id() as usize].origin()
            });
        let vector = global.origin() - parent_pos;
        global.with_origin(vector.to_point())
    }

    pub fn propagate_event<'b, W: Widget<C> + 'b>(
        &mut self,
        event: WidgetEvent,
        iter: impl Iterator<Item = &'b mut W>,
    ) {
        for w in iter {
            let id = w.id();

            if let Some(point) = event.get_point() {
                let child_pos = self.global_positions[id.widget_id() as usize];
                if child_pos.contains(point) {
                    w.event(event, self);
                    return;
                }
            }

            w.event(event.clone(), self);
        }
    }

    pub fn unwrap(self) -> bool {
        self.resize
    }

    pub fn set_active(&mut self, id: WidgetID, active: bool) {
        if let Some(old_id) = *self.active_widget {
            if !active && old_id != id {
                return;
            }
            if !(active && old_id == id) {
                let resize = self.component.event(
                    old_id,
                    WidgetEvent::ActiveChange,
                    self.handle,
                    self.global_positions,
                    self.active_widget,
                    self.hovered_widgets,
                );
                if resize {
                    self.resize = true;
                }
            }
        }
        *self.active_widget = active.then_some(id);
    }

    pub fn add_hover(&mut self, id: WidgetID) -> bool {
        if !self.hovered_widgets.contains(&id) {
            self.hovered_widgets.push(id);
            true
        } else {
            false
        }
    }

    pub fn is_active(&self, id: WidgetID) -> bool {
        self.active_widget == &Some(id)
    }

    pub fn is_hovered(&self, id: WidgetID) -> bool {
        self.hovered_widgets.contains(&id)
    }

    pub fn get_handler(&mut self) -> &mut C::Handler {
        self.component.get_handler()
    }
}
