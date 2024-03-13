#[allow(clippy::suspicious_else_formatting)]
#[allow(clippy::collapsible_if)]
#[allow(clippy::match_single_binding)]
#[allow(unused_imports)]
mod gen {
    use super::__private_CompStruct as CompStruct;
    use gui::gui_core::vello::SceneBuilder;
    use gui::gui_core::widget::{
        EventHandle, Handle, RenderHandle, ResizeHandle, RuntimeID, UpdateHandle, Widget,
        WidgetEvent, WidgetID,
    };
    use gui::gui_core::{
        Component, ComponentHolder, ComponentTypeInfo, LayoutConstraints, MultiComponent, Size,
        ToComponent, ToHandler, Update, Variable,
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
                WidgetSet0::W0(_) => WidgetID::new(1u32),
                WidgetSet0::W1(_) => WidgetID::new(2u32),
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
    pub struct MultiComponentHolder {
        #[allow(dead_code)]
        parent_id: RuntimeID,
        light_holder: <<crate::__gui_private::TrafficLight as ComponentTypeInfo>::ToComponent as ToComponent>::Component,
    }

    #[automatically_derived]
    impl MultiComponentHolder {
        pub fn new(comp: &mut CompStruct, parent_id: RuntimeID) -> Self {
            let comp_holder = <CompStruct as ComponentHolder<light>>::comp_holder(comp);
            let light_holder = comp_holder
                .take()
                .expect("Component is initialised.")
                .to_component_holder(RuntimeID::next());
            Self {
                parent_id,
                light_holder,
            }
        }
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
                id if id == self.light_holder.id() => self.light_holder.render(scene, handle),
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
                id if id == self.light_holder.id() => {
                    self.light_holder.update_vars(force_update, handle)
                }
                _ => false,
            }
        }
        #[allow(clippy::nonminimal_bool)]
        fn force_update_vars(&mut self, handle: &mut Handle) -> bool {
            let light_holder = self.light_holder.update_vars(true, handle);
            false || light_holder
        }
        fn resize(
            &mut self,
            runtime_id: RuntimeID,
            constraints: LayoutConstraints,
            handle: &mut Handle,
        ) -> Size {
            match runtime_id {
                id if id == self.light_holder.id() => self.light_holder.resize(constraints, handle),
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
                id if id == self.light_holder.id() => {
                    self.light_holder.propagate_event(event, handle)
                }
                _ => false,
            }
        }
        #[allow(clippy::nonminimal_bool)]
        fn event(
            &mut self,
            runtime_id: RuntimeID,
            widget_id: WidgetID,
            event: WidgetEvent,
            handle: &mut Handle,
        ) -> bool {
            let light_holder =
                self.light_holder
                    .event(runtime_id, widget_id, event.clone(), handle);
            false || light_holder
        }
        fn get_parent(
            &self,
            runtime_id: RuntimeID,
            widget_id: WidgetID,
        ) -> Option<(RuntimeID, WidgetID)> {
            if widget_id.id() == 0 {
                if self.light_holder.id() == runtime_id {
                    return Some((self.parent_id, WidgetID::new(2u32)));
                }
            }
            self.light_holder.get_parent(runtime_id, widget_id)
        }
        fn get_id(&self, name: &str) -> Option<(RuntimeID, WidgetID)> {
            self.light_holder.get_id(name)
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
        runtime_id: RuntimeID,
        widget: ::gui::gui_widget::HVStack<CompStruct, WidgetSet0>,
        multi_comp: MultiComponentHolder,
    }
    #[automatically_derived]
    impl ToComponent for CompStruct {
        type Component = HolderHolder;
        type HeldComponents = MultiComponentHolder;
        fn to_component_holder(mut self, runtime_id: RuntimeID) -> Self::Component {
            HolderHolder {
                widget: ::gui::gui_widget::HVStack::new_horizontal(
                    WidgetID::new(0u32),
                    vec![
                        WidgetSet0::W0(::gui::gui_widget::Text::new(WidgetID::new(1u32))),
                        WidgetSet0::W1(::gui::gui_widget::CompHolder::new(WidgetID::new(2u32))),
                    ],
                ),
                runtime_id,
                multi_comp: MultiComponentHolder::new(&mut self, runtime_id),
                comp_struct: self,
            }
        }
        fn get_parent(&self, widget_id: WidgetID) -> Option<WidgetID> {
            match widget_id.id() {
                1u32 | 2u32 => Some(WidgetID::new(0u32)),
                _ => None,
            }
        }
        fn get_id(&self, name: &str) -> Option<WidgetID> {
            match name {
                "CompHolder" => Some(WidgetID::new(2u32)),
                "Text" => Some(WidgetID::new(1u32)),
                _ => None,
            }
        }
    }
    #[automatically_derived]
    impl Component for HolderHolder {
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
            let need_multi_comp_resize = self.multi_comp.force_update_vars(handle);
            let mut update_handle = UpdateHandle::new(handle, self.runtime_id);
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
                let value = self.multi_comp.light_holder.id();
                widget.set_child(value, handle_ref);
            }
            if force_update {
                let value = get_bundle_message("Holder-Text-text", None);
                let widget = &mut self.widget.widgets(0usize).w0();
                widget.set_text(value, handle_ref);
            }
            update_handle.unwrap() || need_multi_comp_resize
        }
        fn resize(&mut self, constraints: LayoutConstraints, handle: &mut Handle) -> Size {
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
                self.comp_struct
                    .get_parent(widget_id)
                    .map(|id| (self.runtime_id, id))
            }
        }
        fn get_id(&self, name: &str) -> Option<(RuntimeID, WidgetID)> {
            self.comp_struct.get_id(name).map_or_else(
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
