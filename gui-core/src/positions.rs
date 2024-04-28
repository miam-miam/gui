use crate::widget::{RuntimeID, WidgetID};
use crate::{Component, Point};
use glazier::kurbo::Rect;
use itertools::Itertools;
use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct WidgetInfo {
    pos_map: HashMap<RuntimeID, Vec<Rect>>,
    active_widget: Option<(RuntimeID, WidgetID)>,
    hovered_widgets: Vec<(RuntimeID, WidgetID)>,
}

impl WidgetInfo {
    pub fn reset_positions(&mut self) {
        self.pos_map.clear();
    }
    pub fn get_rect(&self, runtime_id: RuntimeID, widget_id: WidgetID) -> Rect {
        self.pos_map
            .get(&runtime_id)
            .and_then(|v| v.get(widget_id.id() as usize))
            .copied()
            .unwrap_or_default()
    }

    pub fn get_parent_rect(&self) -> Rect {
        let parent_runtime_id = self.pos_map.keys().min().copied().unwrap_or_default();
        self.get_rect(parent_runtime_id, WidgetID::new(0))
    }

    /// Covers the positions of all widgets from their local positions
    /// (positioned in their parent's local space) to global window positions by mutating the position map.
    ///
    /// # Arguments
    ///
    /// * `rect`: The rect of the first component's widget, this will normally be a rect that positions the widget in the window's space.
    /// * `component`: The main component used to run the application, this is used to get the parent of each widget.
    pub fn convert_to_global_positions<C: Component>(&mut self, rect: Rect, component: &C) {
        let parent_runtime_id = self.pos_map.keys().min().copied().unwrap_or_default();
        self.position_widget(parent_runtime_id, WidgetID::new(0), rect);
        let local_positions = self.pos_map.clone();
        self.reset_positions();
        self.position_widget(parent_runtime_id, WidgetID::new(0), rect);

        for (runtime_id, positions) in local_positions.into_iter().sorted_by_key(|(r, _)| *r) {
            for (id, rect) in positions.into_iter().enumerate() {
                let widget_id = WidgetID::new(id as u32);
                if let Some((runtime_parent, widget_parent)) =
                    component.get_parent(runtime_id, widget_id)
                {
                    let parent_rect = self.get_rect(runtime_parent, widget_parent);
                    self.position_widget(
                        runtime_id,
                        widget_id,
                        rect + parent_rect.origin().to_vec2(),
                    );
                }
            }
        }
    }

    pub fn position_widget(&mut self, runtime_id: RuntimeID, child_id: WidgetID, rect: Rect) {
        let array = self.pos_map.entry(runtime_id).or_default();
        if array.len() <= child_id.id() as usize {
            array.resize(child_id.id() as usize + 1, Rect::ZERO);
        }
        array[child_id.id() as usize] = rect;
    }

    pub fn remove_runtime_id(&mut self, runtime_id: RuntimeID) {
        self.pos_map.remove(&runtime_id);
        if let Some((active_id, _)) = self.active_widget {
            if active_id == runtime_id {
                self.active_widget = None;
            }
        }
        self.hovered_widgets
            .retain(|(hovered_id, _)| *hovered_id != runtime_id);
    }

    pub fn is_active(&self, runtime_id: RuntimeID, widget_id: WidgetID) -> bool {
        self.active_widget
            .is_some_and(|(r_id, w_id)| r_id == runtime_id && w_id == widget_id)
    }
    pub fn set_active_widget(&mut self) -> &mut Option<(RuntimeID, WidgetID)> {
        &mut self.active_widget
    }

    pub fn get_active_widget(&self) -> Option<(RuntimeID, WidgetID)> {
        self.active_widget
    }

    pub fn is_hovered(&self, runtime_id: RuntimeID, widget_id: WidgetID) -> bool {
        self.hovered_widgets.contains(&(runtime_id, widget_id))
    }

    pub fn add_hover(&mut self, runtime_id: RuntimeID, widget_id: WidgetID) -> bool {
        if !self.is_hovered(runtime_id, widget_id) {
            self.hovered_widgets.push((runtime_id, widget_id));
            true
        } else {
            false
        }
    }

    pub fn remove_un_hovered(&mut self, mouse_point: Point) -> Vec<(RuntimeID, WidgetID)> {
        let (hovered, un_hovered) = self
            .hovered_widgets
            .iter()
            .partition(|i| self.get_rect(i.0, i.1).contains(mouse_point));
        self.hovered_widgets = hovered;
        un_hovered
    }
}

#[cfg(test)]
mod tests {
    use super::WidgetInfo;
    use crate::handles::Handle;
    use crate::widget::{RuntimeID, WidgetEvent, WidgetID};
    use crate::{Component, LayoutConstraints, Size};
    use glazier::kurbo::{Point, Rect};
    use std::any::Any;
    use std::collections::HashMap;
    use vello::SceneBuilder;

    const WIDGET_ZERO: WidgetID = WidgetID::new(0);
    const RUNTIME_ZERO: RuntimeID = RuntimeID::new(0);

    #[derive(Default, Clone)]
    struct ComponentMock {
        parent: HashMap<(RuntimeID, WidgetID), (RuntimeID, WidgetID)>,
    }

    impl Component for ComponentMock {
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

        fn get_parent(
            &self,
            runtime_id: RuntimeID,
            widget_id: WidgetID,
        ) -> Option<(RuntimeID, WidgetID)> {
            self.parent.get(&(runtime_id, widget_id)).copied()
        }

        fn get_id(&self, _name: &str) -> Option<(RuntimeID, WidgetID)> {
            unimplemented!()
        }

        fn get_comp_struct(&mut self) -> &mut dyn Any {
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

        fn id(&self) -> RuntimeID {
            unimplemented!()
        }
    }

    #[test]
    fn positions_are_reset() {
        let mut widget_info = WidgetInfo::default();
        widget_info.position_widget(RUNTIME_ZERO, WIDGET_ZERO, Rect::new(0.0, 0.0, 10.0, 10.0));
        widget_info.reset_positions();
        assert!(widget_info.pos_map.is_empty());
    }

    #[test]
    fn get_rect_returns() {
        let mut widget_info = WidgetInfo::default();
        let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        widget_info.position_widget(RUNTIME_ZERO, WIDGET_ZERO, rect);
        let result = widget_info.get_rect(RUNTIME_ZERO, WIDGET_ZERO);
        assert_eq!(result, rect);
    }

    #[test]
    fn get_rect_defaults() {
        let widget_info = WidgetInfo::default();
        let result = widget_info.get_rect(RUNTIME_ZERO, WIDGET_ZERO);
        assert_eq!(result, Rect::ZERO);
    }

    #[test]
    fn active_widget() {
        let mut widget_info = WidgetInfo::default();
        assert_eq!(widget_info.get_active_widget(), None);
        let active_widget = (RUNTIME_ZERO, WIDGET_ZERO);
        *widget_info.set_active_widget() = Some(active_widget);
        assert_eq!(widget_info.get_active_widget(), Some(active_widget));
        assert!(widget_info.is_active(active_widget.0, active_widget.1));
    }

    #[test]
    fn hovered_widget() {
        let mut widget_info = WidgetInfo::default();
        assert!(!widget_info.is_hovered(RUNTIME_ZERO, WIDGET_ZERO));
        widget_info.add_hover(RUNTIME_ZERO, WIDGET_ZERO);
        assert!(widget_info.is_hovered(RUNTIME_ZERO, WIDGET_ZERO));
    }

    #[test]
    fn remove_un_hovered() {
        let mut widget_info = WidgetInfo::default();
        widget_info.add_hover(RUNTIME_ZERO, WIDGET_ZERO);
        widget_info.add_hover(RuntimeID::new(1), WIDGET_ZERO);
        widget_info.position_widget(RUNTIME_ZERO, WIDGET_ZERO, Rect::new(0.0, 0.0, 10.0, 10.0));
        widget_info.position_widget(
            RuntimeID::new(1),
            WIDGET_ZERO,
            Rect::new(20.0, 20.0, 30.0, 30.0),
        );
        let un_hovered = widget_info.remove_un_hovered(Point::new(5.0, 5.0));
        assert_eq!(un_hovered, vec![(RuntimeID::new(1), WIDGET_ZERO)]);
        assert!(widget_info.is_hovered(RUNTIME_ZERO, WIDGET_ZERO));
    }

    #[test]
    fn global_position_of_empty_position_map() {
        let mut widget_info = WidgetInfo::default();
        let component = ComponentMock::default();
        let rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        widget_info.convert_to_global_positions(rect, &component);
        assert!(widget_info.pos_map.is_empty());
    }

    #[test]
    fn convert_to_global_position() {
        let mut widget_info = WidgetInfo::default();
        let mut component = ComponentMock::default();
        let child_widget_id = WidgetID::new(1);
        component
            .parent
            .insert((RUNTIME_ZERO, child_widget_id), (RUNTIME_ZERO, WIDGET_ZERO));

        let window_rect = Rect::new(10.0, 10.0, 20.0, 20.0);
        let parent_rect = Rect::new(0.0, 0.0, 10.0, 10.0);
        let child_rect = Rect::new(1.0, 2.0, 5.0, 5.0);

        widget_info.position_widget(RUNTIME_ZERO, WIDGET_ZERO, parent_rect);
        widget_info.position_widget(RUNTIME_ZERO, child_widget_id, child_rect);

        widget_info.convert_to_global_positions(window_rect, &component);

        assert_eq!(
            widget_info.get_rect(RUNTIME_ZERO, child_widget_id),
            Rect::new(11.0, 12.0, 15.0, 15.0)
        );
        assert_eq!(widget_info.get_rect(RUNTIME_ZERO, WIDGET_ZERO), window_rect);
    }
}
