#[allow(clippy::suspicious_else_formatting)]
#[allow(clippy::collapsible_if)]
mod gen {
    use super::__private_CompStruct as CompStruct;
    use gui::gui_core::glazier::kurbo::Rect;
    use gui::gui_core::vello::SceneBuilder;
    use gui::gui_core::widget::{
        EventHandle, Handle, RenderHandle, ResizeHandle, UpdateHandle, Widget, WidgetEvent,
        WidgetID,
    };
    use gui::gui_core::{
        Component, ComponentHolder, ComponentTypeInfo, LayoutConstraints, Size, ToComponent,
        ToHandler, Update, Variable,
    };
    use std::any::Any;
    enum WidgetSet0 {
        W0(::gui::gui_widget::Text),
        W1(::gui::gui_widget::CompHolder),
    }
    impl WidgetSet0 {
        pub fn w0(&mut self) -> &mut ::gui::gui_widget::Text {
            if let WidgetSet0::W0(val) = self {
                val
            } else {
                panic!("Incorrect wrapped type.")
            }
        }
        pub fn w1(&mut self) -> &mut ::gui::gui_widget::CompHolder {
            if let WidgetSet0::W1(val) = self {
                val
            } else {
                panic!("Incorrect wrapped type.")
            }
        }
    }
    impl Widget<CompStruct> for WidgetSet0 {
        fn id(&self) -> WidgetID {
            match self {
                WidgetSet0::W0(_) => WidgetID::new(0u32, 1u32),
                WidgetSet0::W1(_) => WidgetID::new(0u32, 2u32),
            }
        }
        fn render(&mut self, scene: &mut SceneBuilder, handle: &mut RenderHandle<CompStruct>) {
            match self {
                WidgetSet0::W0(w) => {
                    <::gui::gui_widget::Text as Widget<CompStruct>>::render(w, scene, handle)
                }
                WidgetSet0::W1(w) => {
                    <::gui::gui_widget::CompHolder as Widget<CompStruct>>::render(w, scene, handle)
                }
            }
        }
        fn resize(
            &mut self,
            constraints: LayoutConstraints,
            handle: &mut ResizeHandle<CompStruct>,
        ) -> Size {
            match self {
                WidgetSet0::W0(w) => {
                    <::gui::gui_widget::Text as Widget<CompStruct>>::resize(w, constraints, handle)
                }
                WidgetSet0::W1(w) => <::gui::gui_widget::CompHolder as Widget<CompStruct>>::resize(
                    w,
                    constraints,
                    handle,
                ),
            }
        }
        fn event(&mut self, event: WidgetEvent, handle: &mut EventHandle<CompStruct>) {
            match self {
                WidgetSet0::W0(w) => {
                    <::gui::gui_widget::Text as Widget<CompStruct>>::event(w, event, handle)
                }
                WidgetSet0::W1(w) => {
                    <::gui::gui_widget::CompHolder as Widget<CompStruct>>::event(w, event, handle)
                }
            }
        }
    }
    #[allow(non_camel_case_types)]
    pub(crate) struct light;
    impl Variable for light {
        type VarType = <crate::__gui_private::TrafficLight as ComponentTypeInfo>::ToComponent;
    }
    impl ComponentTypeInfo for crate::__gui_private::Holder {
        type ToComponent = CompStruct;
    }
    pub struct MultiComponent {
        light_holder: <<crate::__gui_private::TrafficLight as ComponentTypeInfo>::ToComponent as ToComponent>::Component,
    }
    impl MultiComponent {
        pub fn new(comp: &mut CompStruct) -> Self {
            let comp_holder = <CompStruct as ComponentHolder<light>>::comp_holder(comp);
            let light_holder = comp_holder
                .take()
                .expect("Component is initialised.")
                .to_component_holder();
            Self { light_holder }
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
                3u32 => self.light_holder.render(
                    scene,
                    handle,
                    global_positions,
                    active_widget,
                    hovered_widgets,
                ),
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
                3u32 => self
                    .light_holder
                    .update_vars(force_update, handle, global_positions),
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
                3u32 => self
                    .light_holder
                    .resize(constraints, handle, local_positions),
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
                3u32 => self.light_holder.propagate_event(
                    event,
                    handle,
                    global_positions,
                    active_widget,
                    hovered_widgets,
                ),
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
                3u32 => self.light_holder.event(
                    id,
                    event,
                    handle,
                    global_positions,
                    active_widget,
                    hovered_widgets,
                ),
                _ => false,
            }
        }
    }
    use gui::{FluentArgs, FluentBundle, FluentResource};
    use std::borrow::Cow;
    fn get_bundle_message<'a>(message: &'a str, args: Option<&'a FluentArgs<'_>>) -> Cow<'a, str> {
        use gui::langid;
        use std::sync::OnceLock;
        static BUNDLE: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();
        const FTL_STRING: &str = include_str!(concat!(env!("OUT_DIR"), "/Holder.ftl"));
        let mut errors = vec![];
        let bundle = BUNDLE.get_or_init(|| {
            let mut bundle = FluentBundle::new_concurrent(vec![langid!("en-GB")]);
            let resource =
                FluentResource::try_new(FTL_STRING.to_string()).expect("FTL string is valid.");
            bundle
                .add_resource(resource)
                .expect("No identifiers are overlapping.");
            bundle
        });
        let message = bundle.get_message(message).expect("Message exists.");
        let pattern = message.value().expect("Value exists.");
        bundle.format_pattern(pattern, args, &mut errors)
    }
    #[allow(non_snake_case)]
    pub struct HolderHolder {
        comp_struct: CompStruct,
        widget: ::gui::gui_widget::HVStack<CompStruct, WidgetSet0>,
        multi_comp: MultiComponent,
    }
    #[automatically_derived]
    impl ToComponent for CompStruct {
        type Component = HolderHolder;
        type HeldComponents = MultiComponent;
        fn to_component_holder(mut self) -> Self::Component {
            HolderHolder {
                widget: ::gui::gui_widget::HVStack::new_horizontal(
                    WidgetID::new(0u32, 0u32),
                    vec![
                        WidgetSet0::W0(::gui::gui_widget::Text::new(WidgetID::new(0u32, 1u32))),
                        WidgetSet0::W1(::gui::gui_widget::CompHolder::new(WidgetID::new(
                            0u32, 2u32,
                        ))),
                    ],
                ),
                multi_comp: MultiComponent::new(&mut self),
                comp_struct: self,
            }
        }
        fn get_parent(&self, id: WidgetID) -> Option<WidgetID> {
            match (id.component_id(), id.widget_id()) {
                (0u32, 1u32) | (0u32, 2u32) => Some(WidgetID::new(0u32, 0u32)),
                _ => None,
            }
        }
        fn get_id(&self, name: &str) -> Option<WidgetID> {
            match name {
                "CompHolder" => Some(WidgetID::new(0u32, 2u32)),
                "Text" => Some(WidgetID::new(0u32, 1u32)),
                _ => None,
            }
        }
    }
    #[automatically_derived]
    impl Component for HolderHolder {
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
            if force_update {
                let widget = &mut self.widget;
                let value = 10f32;
                widget.set_spacing(value, handle_ref);
                let widget = &mut self.widget.widgets(0usize).w0();
                let value = ::gui::gui_core::Colour::rgba8(33u8, 37u8, 41u8, 255u8);
                widget.set_colour(value, handle_ref);
                let value = 14f32;
                widget.set_size(value, handle_ref);
                let widget = &mut self.widget.widgets(1usize).w1();
                let value = WidgetID::new(0u32, 3u32);
                widget.set_child(value, handle_ref);
            }
            if force_update {
                let value = get_bundle_message("Holder-Text-text", None);
                let widget = &mut self.widget.widgets(0usize).w0();
                widget.set_text(value, handle_ref);
            }
            update_handle.unwrap()
        }
        fn resize<'a>(
            &mut self,
            constraints: LayoutConstraints,
            handle: &'a mut Handle,
            local_positions: &'a mut [Rect],
        ) -> Size {
            let mut resize_handle =
                ResizeHandle::new(handle, local_positions, &mut self.comp_struct);
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
                if self.event(
                    id,
                    e,
                    handle,
                    global_positions,
                    active_widget,
                    hovered_widgets,
                ) {
                    resize = true;
                }
            }
            resize
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
                (0u32, 1u32) => {
                    self.widget.widgets(0usize).w0().event(event, handle_ref);
                }
                (0u32, 2u32) => {
                    self.widget.widgets(1usize).w1().event(event, handle_ref);
                }
                (0u32, 0u32) => {
                    self.widget.event(event, handle_ref);
                }
                _ => {}
            }
            let (mut resize, events) = event_handle.unwrap();
            for (id, e) in events {
                if self.event(
                    id,
                    e,
                    handle,
                    global_positions,
                    active_widget,
                    hovered_widgets,
                ) {
                    resize = true;
                }
            }
            resize
        }
    }
}
