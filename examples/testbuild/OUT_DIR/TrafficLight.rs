#[allow(clippy::suspicious_else_formatting)]
#[allow(clippy::collapsible_if)]
#[allow(clippy::match_single_binding)]
#[allow(unused_imports)]
mod gen {
    use super::__private_CompStruct as CompStruct;
    use std::any::Any;
    use gui::gui_core::vello::SceneBuilder;
    use gui::gui_core::widget::{
        RuntimeID, Widget, WidgetID, RenderHandle, ResizeHandle, EventHandle,
        UpdateHandle, WidgetEvent, Handle,
    };
    use gui::gui_core::{
        Component, ComponentHolder, ComponentTypeInfo, LayoutConstraints, MultiComponent,
        Size, ToComponent, ToHandler, Update, Variable,
    };
    #[allow(non_camel_case_types)]
    #[derive(Default, Copy, Clone, Eq, PartialEq)]
    pub(crate) enum State {
        #[default]
        Red,
        RedYellow,
        Green,
        Yellow,
    }
    #[allow(non_camel_case_types)]
    pub(crate) struct state;
    impl Variable for state {
        type VarType = State;
    }
    enum WidgetSet1 {
        W0(::gui::gui_widget::ImageWidget),
        W1(::gui::gui_widget::ImageWidget),
        W2(::gui::gui_widget::ImageWidget),
    }
    impl WidgetSet1 {
        pub fn w0(&mut self) -> &mut ::gui::gui_widget::ImageWidget {
            if let WidgetSet1::W0(val) = self {
                val
            } else {
                panic!("Incorrect wrapped type.")
            }
        }
        pub fn w1(&mut self) -> &mut ::gui::gui_widget::ImageWidget {
            if let WidgetSet1::W1(val) = self {
                val
            } else {
                panic!("Incorrect wrapped type.")
            }
        }
        pub fn w2(&mut self) -> &mut ::gui::gui_widget::ImageWidget {
            if let WidgetSet1::W2(val) = self {
                val
            } else {
                panic!("Incorrect wrapped type.")
            }
        }
    }
    impl Widget<CompStruct> for WidgetSet1 {
        fn id(&self) -> WidgetID {
            match self {
                WidgetSet1::W0(_) => WidgetID::new(1u32),
                WidgetSet1::W1(_) => WidgetID::new(2u32),
                WidgetSet1::W2(_) => WidgetID::new(3u32),
            }
        }
        fn render(
            &mut self,
            scene: &mut SceneBuilder,
            handle: &mut RenderHandle<CompStruct>,
        ) {
            match self {
                WidgetSet1::W0(w) => {
                    <::gui::gui_widget::ImageWidget as Widget<
                        CompStruct,
                    >>::render(w, scene, handle)
                }
                WidgetSet1::W1(w) => {
                    <::gui::gui_widget::ImageWidget as Widget<
                        CompStruct,
                    >>::render(w, scene, handle)
                }
                WidgetSet1::W2(w) => {
                    <::gui::gui_widget::ImageWidget as Widget<
                        CompStruct,
                    >>::render(w, scene, handle)
                }
            }
        }
        fn resize(
            &mut self,
            constraints: LayoutConstraints,
            handle: &mut ResizeHandle<CompStruct>,
        ) -> Size {
            match self {
                WidgetSet1::W0(w) => {
                    <::gui::gui_widget::ImageWidget as Widget<
                        CompStruct,
                    >>::resize(w, constraints, handle)
                }
                WidgetSet1::W1(w) => {
                    <::gui::gui_widget::ImageWidget as Widget<
                        CompStruct,
                    >>::resize(w, constraints, handle)
                }
                WidgetSet1::W2(w) => {
                    <::gui::gui_widget::ImageWidget as Widget<
                        CompStruct,
                    >>::resize(w, constraints, handle)
                }
            }
        }
        fn event(&mut self, event: WidgetEvent, handle: &mut EventHandle<CompStruct>) {
            match self {
                WidgetSet1::W0(w) => {
                    <::gui::gui_widget::ImageWidget as Widget<
                        CompStruct,
                    >>::event(w, event, handle)
                }
                WidgetSet1::W1(w) => {
                    <::gui::gui_widget::ImageWidget as Widget<
                        CompStruct,
                    >>::event(w, event, handle)
                }
                WidgetSet1::W2(w) => {
                    <::gui::gui_widget::ImageWidget as Widget<
                        CompStruct,
                    >>::event(w, event, handle)
                }
            }
        }
    }
    impl ComponentTypeInfo for crate::__gui_private::TrafficLight {
        type ToComponent = CompStruct;
    }
    pub struct MultiComponentHolder {
        #[allow(dead_code)]
        parent_id: RuntimeID,
    }
    #[automatically_derived]
    impl MultiComponentHolder {
        pub fn new(comp: &mut CompStruct, parent_id: RuntimeID) -> Self {
            Self { parent_id }
        }
        pub fn get_messages(&mut self, comp: &mut CompStruct) {}
    }
    #[automatically_derived]
    impl MultiComponent for MultiComponentHolder {
        fn render(
            &mut self,
            runtime_id: RuntimeID,
            scene: &mut SceneBuilder,
            handle: &mut Handle,
        ) -> bool {
            match runtime_id {
                _ => false,
            }
        }
        fn update_vars(
            &mut self,
            runtime_id: RuntimeID,
            force_update: bool,
            handle: &mut Handle,
        ) -> bool {
            match runtime_id {
                _ => false,
            }
        }
        fn update_all_vars(&mut self, force_update: bool, handle: &mut Handle) -> bool {
            false
        }
        fn resize(
            &mut self,
            runtime_id: RuntimeID,
            constraints: LayoutConstraints,
            handle: &mut Handle,
        ) -> Size {
            match runtime_id {
                _ => Size::ZERO,
            }
        }
        fn propagate_event(
            &mut self,
            runtime_id: RuntimeID,
            event: WidgetEvent,
            handle: &mut Handle,
        ) -> bool {
            match runtime_id {
                _ => false,
            }
        }
        fn event(
            &mut self,
            runtime_id: RuntimeID,
            widget_id: WidgetID,
            event: WidgetEvent,
            handle: &mut Handle,
        ) -> bool {
            false
        }
        fn get_parent(
            &self,
            runtime_id: RuntimeID,
            widget_id: WidgetID,
        ) -> Option<(RuntimeID, WidgetID)> {
            None
        }
        fn get_id(&self, name: &str) -> Option<(RuntimeID, WidgetID)> {
            None
        }
    }
    #[allow(non_snake_case)]
    pub struct TrafficLightHolder {
        comp_struct: CompStruct,
        runtime_id: RuntimeID,
        widget: ::gui::gui_widget::HVStack<WidgetSet1>,
        state: State,
        multi_comp: MultiComponentHolder,
    }
    #[automatically_derived]
    impl ToComponent for CompStruct {
        type Component = TrafficLightHolder;
        type HeldComponents = MultiComponentHolder;
        fn to_component_holder(mut self, runtime_id: RuntimeID) -> Self::Component {
            TrafficLightHolder {
                widget: ::gui::gui_widget::HVStack::new_vertical(
                    WidgetID::new(0u32),
                    vec![
                        WidgetSet1::W0(::gui::gui_widget::ImageWidget::new(WidgetID::new(1u32))),
                        WidgetSet1::W1(::gui::gui_widget::ImageWidget::new(WidgetID::new(2u32))),
                        WidgetSet1::W2(::gui::gui_widget::ImageWidget::new(WidgetID::new(3u32)))
                    ],
                ),
                runtime_id,
                multi_comp: MultiComponentHolder::new(&mut self, runtime_id),
                comp_struct: self,
                state: Default::default(),
            }
        }
        #[allow(clippy::manual_range_patterns)]
        fn get_parent(&self, widget_id: WidgetID) -> Option<WidgetID> {
            match widget_id.id() {
                1u32 | 2u32 | 3u32 => Some(WidgetID::new(0u32)),
                _ => None,
            }
        }
        fn get_id(&self, name: &str) -> Option<WidgetID> {
            match name {
                "VStack" => Some(WidgetID::new(0u32)),
                "GreenLight" => Some(WidgetID::new(3u32)),
                "YellowLight" => Some(WidgetID::new(2u32)),
                "RedLight" => Some(WidgetID::new(1u32)),
                _ => None,
            }
        }
    }
    impl TrafficLightHolder {
        #[allow(dead_code)]
        pub fn comp_struct(&mut self) -> &mut CompStruct {
            &mut self.comp_struct
        }
    }
    #[automatically_derived]
    impl Component for TrafficLightHolder {
        fn render(&mut self, scene: &mut SceneBuilder, handle: &mut Handle) -> bool {
            let mut render_handle = RenderHandle::new(
                handle,
                self.runtime_id,
                &mut self.comp_struct,
                &mut self.multi_comp,
            );
            self.widget.render(scene, &mut render_handle);
            render_handle.unwrap()
        }
        #[allow(unused_mut)]
        fn update_vars(&mut self, mut force_update: bool, handle: &mut Handle) -> bool {
            self.multi_comp.get_messages(&mut self.comp_struct);
            let need_multi_comp_resize = self
                .multi_comp
                .update_all_vars(force_update, handle);
            let mut update_handle = UpdateHandle::new(handle, self.runtime_id);
            let handle_ref = &mut update_handle;
            if force_update
                || <CompStruct as Update<state>>::is_updated(&self.comp_struct)
            {
                let new_state = <CompStruct as Update<state>>::value(&self.comp_struct);
                if self.state != new_state {
                    self.state = new_state;
                    force_update = true;
                }
            }
            if force_update {
                let widget = &mut self.widget;
                let value = 10f32;
                widget.set_spacing(value, handle_ref);
                if self.state != State::Red && self.state != State::RedYellow {
                    let widget = &mut self.widget.widgets(0usize).w0();
                    let value = "./res/red_off.png";
                    widget.set_image_from_file(value, handle_ref);
                }
                if self.state == State::Red || self.state == State::RedYellow {
                    let widget = &mut self.widget.widgets(0usize).w0();
                    let value = "./res/red_on.png";
                    widget.set_image_from_file(value, handle_ref);
                }
                if self.state != State::RedYellow && self.state != State::Yellow {
                    let widget = &mut self.widget.widgets(1usize).w1();
                    let value = "./res/yellow_off.png";
                    widget.set_image_from_file(value, handle_ref);
                }
                if self.state == State::RedYellow || self.state == State::Yellow {
                    let widget = &mut self.widget.widgets(1usize).w1();
                    let value = "./res/yellow_on.png";
                    widget.set_image_from_file(value, handle_ref);
                }
                if self.state != State::Green {
                    let widget = &mut self.widget.widgets(2usize).w2();
                    let value = "./res/green_off.png";
                    widget.set_image_from_file(value, handle_ref);
                }
                if self.state == State::Green {
                    let widget = &mut self.widget.widgets(2usize).w2();
                    let value = "./res/green_on.png";
                    widget.set_image_from_file(value, handle_ref);
                }
            }
            update_handle.unwrap() || need_multi_comp_resize
        }
        fn resize(
            &mut self,
            constraints: LayoutConstraints,
            handle: &mut Handle,
        ) -> Size {
            let mut resize_handle = ResizeHandle::new(
                handle,
                self.runtime_id,
                &mut self.comp_struct,
                &mut self.multi_comp,
            );
            self.widget.resize(constraints, &mut resize_handle)
        }
        fn propagate_event(&mut self, event: WidgetEvent, handle: &mut Handle) -> bool {
            let mut event_handle = EventHandle::new(
                handle,
                self.runtime_id,
                &mut self.comp_struct,
                &mut self.multi_comp,
            );
            self.widget.event(event, &mut event_handle);
            let (mut resize, events) = event_handle.unwrap();
            for (runtime_id, widget_id, e) in events {
                if self.event(runtime_id, widget_id, e, handle) {
                    resize = true;
                }
            }
            resize
        }
        fn get_parent(
            &self,
            runtime_id: RuntimeID,
            widget_id: WidgetID,
        ) -> Option<(RuntimeID, WidgetID)> {
            if runtime_id != self.runtime_id {
                self.multi_comp.get_parent(runtime_id, widget_id)
            } else {
                self.comp_struct.get_parent(widget_id).map(|id| (self.runtime_id, id))
            }
        }
        fn get_id(&self, name: &str) -> Option<(RuntimeID, WidgetID)> {
            self.comp_struct
                .get_id(name)
                .map_or_else(
                    || self.multi_comp.get_id(name),
                    |id| Some((self.runtime_id, id)),
                )
        }
        fn get_comp_struct(&mut self) -> &mut dyn Any {
            &mut self.comp_struct
        }
        fn event(
            &mut self,
            runtime_id: RuntimeID,
            widget_id: WidgetID,
            event: WidgetEvent,
            handle: &mut Handle,
        ) -> bool {
            if runtime_id != self.runtime_id {
                return self.multi_comp.event(runtime_id, widget_id, event, handle);
            }
            let mut event_handle = EventHandle::new(
                handle,
                self.runtime_id,
                &mut self.comp_struct,
                &mut self.multi_comp,
            );
            let handle_ref = &mut event_handle;
            match widget_id.id() {
                1u32 => {
                    self.widget.widgets(0usize).w0().event(event, handle_ref);
                }
                2u32 => {
                    self.widget.widgets(1usize).w1().event(event, handle_ref);
                }
                3u32 => {
                    self.widget.widgets(2usize).w2().event(event, handle_ref);
                }
                0u32 => {
                    self.widget.event(event, handle_ref);
                }
                _ => {}
            }
            let (mut resize, events) = event_handle.unwrap();
            for (runtime_id, widget_id, e) in events {
                if self.event(runtime_id, widget_id, e, handle) {
                    resize = true;
                }
            }
            resize
        }
        fn id(&self) -> RuntimeID {
            self.runtime_id
        }
    }
}
