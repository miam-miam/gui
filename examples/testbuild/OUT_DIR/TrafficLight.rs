#[allow(clippy::suspicious_else_formatting)]
#[allow(clippy::collapsible_if)]
mod gen {
    use super::__private_CompStruct as CompStruct;
    use std::any::Any;
    use gui::gui_core::vello::SceneBuilder;
    use gui::gui_core::glazier::kurbo::Rect;
    use gui::gui_core::widget::{
        Widget, WidgetID, RenderHandle, ResizeHandle, EventHandle, UpdateHandle,
        WidgetEvent, Handle,
    };
    use gui::gui_core::{
        Component, ComponentHolder, ComponentTypeInfo, LayoutConstraints, Size,
        ToComponent, ToHandler, Update, Variable,
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
    pub struct MultiComponent {}
    impl MultiComponent {
        pub fn new(comp: &mut CompStruct) -> Self {
            Self {}
        }
    }
    impl MultiComponentHolder for MultiComponent {
        fn render(
            &mut self,
            comp_id: WidgetID,
            scene: &mut SceneBuilder,
            handle: &mut Handle,
            global_positions: &mut [Rect],
            active_widget: &mut Option<WidgetID>,
            hovered_widgets: &[WidgetID],
        ) -> bool {
            match id.widget_id() {
                _ => false,
            }
        }
        fn update_vars(
            &mut self,
            comp_id: WidgetID,
            force_update: bool,
            handle: &mut Handle,
            global_positions: &[Rect],
        ) -> bool {
            match id.widget_id() {
                _ => false,
            }
        }
        fn resize(
            &mut self,
            comp_id: WidgetID,
            constraints: LayoutConstraints,
            handle: &mut Handle,
            local_positions: &mut [Rect],
        ) -> Size {
            match id.widget_id() {
                _ => Size::ZERO,
            }
        }
        fn propagate_event(
            &mut self,
            comp_id: WidgetID,
            event: WidgetEvent,
            handle: &mut Handle,
            global_positions: &[Rect],
            active_widget: &mut Option<WidgetID>,
            hovered_widgets: &mut Vec<WidgetID>,
        ) -> bool {
            match id.widget_id() {
                _ => false,
            }
        }
        fn event(
            &mut self,
            comp_id: WidgetID,
            id: WidgetID,
            event: WidgetEvent,
            handle: &mut Handle,
            global_positions: &[Rect],
            active_widget: &mut Option<WidgetID>,
            hovered_widgets: &mut Vec<WidgetID>,
        ) -> bool {
            match id.widget_id() {
                _ => false,
            }
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
        widget: ::gui::gui_widget::Button<Switch, CompStruct, ::gui::gui_widget::Text>,
        state: State,
        multi_comp: MultiComponent,
        TrafficLight_SwitchText_Red_text: FluentArgs<'static>,
    }
    #[automatically_derived]
    impl ToComponent for CompStruct {
        type Component = TrafficLightHolder;
        type HeldComponents = MultiComponent;
        fn to_component_holder(mut self) -> Self::Component {
            TrafficLightHolder {
                widget: ::gui::gui_widget::Button::new(
                    WidgetID::new(1u32, 4u32),
                    ::gui::gui_widget::Text::new(WidgetID::new(1u32, 5u32)),
                ),
                multi_comp: MultiComponent::new(&mut self),
                comp_struct: self,
                state: Default::default(),
                TrafficLight_SwitchText_Red_text: FluentArgs::new(),
            }
        }
        fn largest_id(&self) -> WidgetID {
            WidgetID::new(1u32, 5u32)
        }
        fn get_parent(&self, id: WidgetID) -> Option<WidgetID> {
            match (id.component_id(), id.widget_id()) {
                (1u32, 5u32) => Some(WidgetID::new(1u32, 4u32)),
                _ => None,
            }
        }
        fn get_id(&self, name: &str) -> Option<WidgetID> {
            match name {
                "Switch" => Some(WidgetID::new(1u32, 4u32)),
                "SwitchText" => Some(WidgetID::new(1u32, 5u32)),
                _ => None,
            }
        }
    }
    #[automatically_derived]
    impl Component for TrafficLightHolder {
        fn render<'a>(
            &mut self,
            mut scene: SceneBuilder,
            handle: &'a mut Handle,
            global_positions: &'a mut [Rect],
            active_widget: &'a mut Option<WidgetID>,
            hovered_widgets: &'a [WidgetID],
        ) -> bool {
            let mut render_handle = RenderHandle::new(
                handle,
                global_positions,
                active_widget,
                hovered_widgets,
                &mut self.comp_struct,
            );
            self.widget.render(&mut scene, &mut render_handle);
            render_handle.unwrap()
        }
        #[allow(unused_mut)]
        fn update_vars<'a>(
            &mut self,
            mut force_update: bool,
            handle: &'a mut Handle,
            global_positions: &'a [Rect],
        ) -> bool {
            let mut update_handle = UpdateHandle::new(handle, global_positions);
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
        fn resize<'a>(
            &mut self,
            constraints: LayoutConstraints,
            handle: &'a mut Handle,
            local_positions: &'a mut [Rect],
        ) -> Size {
            let mut resize_handle = ResizeHandle::new(
                handle,
                local_positions,
                &mut self.comp_struct,
            );
            self.widget.resize(constraints, &mut resize_handle)
        }
        fn propagate_event<'a>(
            &mut self,
            event: WidgetEvent,
            handle: &'a mut Handle,
            global_positions: &'a [Rect],
            active_widget: &'a mut Option<WidgetID>,
            hovered_widgets: &'a mut Vec<WidgetID>,
        ) -> bool {
            let mut event_handle = EventHandle::new(
                handle,
                global_positions,
                active_widget,
                hovered_widgets,
                &mut self.comp_struct,
            );
            self.widget.event(event, &mut event_handle);
            let (mut resize, events) = event_handle.unwrap();
            for (id, e) in events {
                if self
                    .event(
                        id,
                        e,
                        handle,
                        global_positions,
                        active_widget,
                        hovered_widgets,
                    )
                {
                    resize = true;
                }
            }
            resize
        }
        fn largest_id(&self) -> WidgetID {
            self.comp_struct.largest_id()
        }
        fn get_parent(&self, id: WidgetID) -> Option<WidgetID> {
            self.comp_struct.get_parent(id)
        }
        fn get_id(&self, name: &str) -> Option<WidgetID> {
            self.comp_struct.get_id(name)
        }
        fn get_comp_struct(&mut self) -> &mut dyn Any {
            &mut self.comp_struct
        }
        fn event<'a>(
            &mut self,
            id: WidgetID,
            event: WidgetEvent,
            handle: &'a mut Handle,
            global_positions: &'a [Rect],
            active_widget: &'a mut Option<WidgetID>,
            hovered_widgets: &'a mut Vec<WidgetID>,
        ) -> bool {
            let mut event_handle = EventHandle::new(
                handle,
                global_positions,
                active_widget,
                hovered_widgets,
                &mut self.comp_struct,
            );
            let handle_ref = &mut event_handle;
            match (id.component_id(), id.widget_id()) {
                (1u32, 5u32) => {
                    self.widget.get_widget().event(event, handle_ref);
                }
                (1u32, 4u32) => {
                    self.widget.event(event, handle_ref);
                }
                _ => {}
            }
            let (mut resize, events) = event_handle.unwrap();
            for (id, e) in events {
                if self
                    .event(
                        id,
                        e,
                        handle,
                        global_positions,
                        active_widget,
                        hovered_widgets,
                    )
                {
                    resize = true;
                }
            }
            resize
        }
    }
}
