#[allow(clippy::suspicious_else_formatting)]
#[allow(clippy::collapsible_if)]
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
        Green,
        Yellow,
        Red,
    }
    #[allow(non_camel_case_types)]
    pub(crate) struct state;
    impl Variable for state {
        type VarType = State;
    }
    #[allow(non_camel_case_types)]
    pub(crate) struct hover_colour;
    impl Variable for hover_colour {
        type VarType = ::gui::gui_core::Colour;
    }
    #[allow(non_camel_case_types)]
    pub(crate) struct count;
    impl Variable for count {
        type VarType = u32;
    }
    pub(crate) struct Switch;
    impl ToHandler for Switch {
        type BaseHandler = CompStruct;
    }
    impl ComponentTypeInfo for crate::__gui_private::TrafficLight {
        type ToComponent = CompStruct;
    }
    pub struct MultiComponentHolder {}
    impl MultiComponentHolder {
        pub fn new(comp: &mut CompStruct) -> Self {
            Self {}
        }
    }
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
    use gui::{FluentBundle, FluentArgs, FluentResource};
    use std::borrow::Cow;
    fn get_bundle_message<'a>(
        message: &'a str,
        args: Option<&'a FluentArgs<'_>>,
    ) -> Cow<'a, str> {
        use std::sync::OnceLock;
        use gui::langid;
        static BUNDLE: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();
        const FTL_STRING: &str = include_str!(
            concat!(env!("OUT_DIR"), "/TrafficLight.ftl")
        );
        let mut errors = vec![];
        let bundle = BUNDLE
            .get_or_init(|| {
                let mut bundle = FluentBundle::new_concurrent(vec![langid!("en-GB")]);
                let resource = FluentResource::try_new(FTL_STRING.to_string())
                    .expect("FTL string is valid.");
                bundle.add_resource(resource).expect("No identifiers are overlapping.");
                bundle
            });
        let message = bundle.get_message(message).expect("Message exists.");
        let pattern = message.value().expect("Value exists.");
        bundle.format_pattern(pattern, args, &mut errors)
    }
    #[allow(non_snake_case)]
    pub struct TrafficLightHolder {
        comp_struct: CompStruct,
        runtime_id: RuntimeID,
        widget: ::gui::gui_widget::Button<Switch, CompStruct, ::gui::gui_widget::Text>,
        state: State,
        multi_comp: MultiComponentHolder,
        TrafficLight_SwitchText_Red_text: FluentArgs<'static>,
    }
    #[automatically_derived]
    impl ToComponent for CompStruct {
        type Component = TrafficLightHolder;
        type HeldComponents = MultiComponentHolder;
        fn to_component_holder(mut self, runtime_id: RuntimeID) -> Self::Component {
            TrafficLightHolder {
                widget: ::gui::gui_widget::Button::new(
                    WidgetID::new(0u32),
                    ::gui::gui_widget::Text::new(WidgetID::new(1u32)),
                ),
                runtime_id,
                multi_comp: MultiComponentHolder::new(&mut self),
                comp_struct: self,
                state: Default::default(),
                TrafficLight_SwitchText_Red_text: FluentArgs::new(),
            }
        }
        fn get_parent(&self, widget_id: WidgetID) -> Option<WidgetID> {
            match widget_id.id() {
                1u32 => Some(WidgetID::new(0u32)),
                _ => None,
            }
        }
        fn get_id(&self, name: &str) -> Option<WidgetID> {
            match name {
                "Switch" => Some(WidgetID::new(0u32)),
                "SwitchText" => Some(WidgetID::new(1u32)),
                _ => None,
            }
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
            let mut update_handle = UpdateHandle::new(handle, self.runtime_id);
            let handle_ref = &mut update_handle;
            let mut text = false;
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
                let value = ::gui::gui_core::Colour::rgba8(206u8, 212u8, 218u8, 255u8);
                widget.set_border_colour(value, handle_ref);
                let value = ::gui::gui_core::Colour::rgba8(248u8, 249u8, 250u8, 255u8);
                widget.set_clicked_colour(value, handle_ref);
                let value = false;
                widget.set_disabled(value, handle_ref);
                let value = ::gui::gui_core::Colour::rgba8(241u8, 243u8, 245u8, 255u8);
                widget.set_disabled_colour(value, handle_ref);
                if self.state == State::Green {
                    let widget = &mut self.widget;
                    let value = ::gui::gui_core::Colour::rgba8(0u8, 128u8, 0u8, 255u8);
                    widget.set_background_colour(value, handle_ref);
                }
                if self.state == State::Yellow {
                    let widget = &mut self.widget;
                    let value = ::gui::gui_core::Colour::rgba8(255u8, 255u8, 0u8, 255u8);
                    widget.set_background_colour(value, handle_ref);
                    let value = ::gui::gui_core::Colour::rgba8(255u8, 255u8, 0u8, 255u8);
                    widget.set_hover_colour(value, handle_ref);
                }
                if self.state == State::Red {
                    let widget = &mut self.widget;
                    let value = ::gui::gui_core::Colour::rgba8(255u8, 0u8, 0u8, 255u8);
                    widget.set_background_colour(value, handle_ref);
                    let value = ::gui::gui_core::Colour::rgba8(255u8, 0u8, 0u8, 255u8);
                    widget.set_hover_colour(value, handle_ref);
                }
                let widget = &mut self.widget.get_widget();
                let value = ::gui::gui_core::Colour::rgba8(33u8, 37u8, 41u8, 255u8);
                widget.set_colour(value, handle_ref);
                let value = 14f32;
                widget.set_size(value, handle_ref);
            }
            if force_update
                || <CompStruct as Update<hover_colour>>::is_updated(&self.comp_struct)
            {
                let value = <CompStruct as Update<
                    hover_colour,
                >>::value(&self.comp_struct);
                if self.state == State::Green {
                    let widget = &mut self.widget;
                    widget.set_hover_colour(value, handle_ref);
                }
            }
            if force_update
                || <CompStruct as Update<count>>::is_updated(&self.comp_struct)
            {
                let value = <CompStruct as Update<count>>::value(&self.comp_struct);
                if self.state == State::Red {
                    text = true;
                    self.TrafficLight_SwitchText_Red_text.set("count", value);
                }
            }
            if self.state == State::Green {
                if force_update {
                    let value = get_bundle_message(
                        "TrafficLight-SwitchText-Green-text",
                        None,
                    );
                    let widget = &mut self.widget.get_widget();
                    widget.set_text(value, handle_ref);
                }
            }
            if self.state == State::Yellow {
                if force_update {
                    let value = get_bundle_message(
                        "TrafficLight-SwitchText-Yellow-text",
                        None,
                    );
                    let widget = &mut self.widget.get_widget();
                    widget.set_text(value, handle_ref);
                }
            }
            if self.state == State::Red {
                if force_update || text {
                    let value = get_bundle_message(
                        "TrafficLight-SwitchText-Red-text",
                        Some(&self.TrafficLight_SwitchText_Red_text),
                    );
                    let widget = &mut self.widget.get_widget();
                    widget.set_text(value, handle_ref);
                }
            }
            <CompStruct as Update<hover_colour>>::reset(&mut self.comp_struct);
            <CompStruct as Update<count>>::reset(&mut self.comp_struct);
            update_handle.unwrap()
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
                    self.widget.get_widget().event(event, handle_ref);
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
