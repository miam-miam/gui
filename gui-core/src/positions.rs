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

    pub fn convert_to_global_positions<C: Component>(&mut self, rect: Rect, component: &C) {
        self.position_widget(RuntimeID::new(0), WidgetID::new(0), rect);
        let local_positions = self.pos_map.clone();
        self.reset_positions();
        self.position_widget(RuntimeID::new(0), WidgetID::new(0), rect);

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
