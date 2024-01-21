#[allow(clippy::suspicious_else_formatting)]
mod gen {
    use super::__private_CompStruct as CompStruct;
    use gui::gui_core::glazier::{PointerEvent, WindowHandle};
    use gui::gui_core::parley::font::FontContext;
    use gui::gui_core::vello::SceneBuilder;
    use gui::gui_core::widget::Widget;
    use gui::gui_core::{Component, ToComponent, ToHandler, Update, Variable};
    use gui::{FluentArgs, FluentBundle, FluentResource};
    use std::borrow::Cow;
    fn get_bundle_message<'a>(message: &'a str, args: Option<&'a FluentArgs<'_>>) -> Cow<'a, str> {
        use gui::langid;
        use std::sync::OnceLock;
        static BUNDLE: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();
        const FTL_STRING: &str = include_str!(concat!(env!("OUT_DIR"), "/Counter.ftl"));
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
    #[allow(non_camel_case_types)]
    pub(crate) struct count;
    impl Variable for count {
        type VarType = f32;
    }
    #[allow(non_camel_case_types)]
    pub(crate) struct disabled;
    impl Variable for disabled {
        type VarType = bool;
    }
    pub(crate) struct Count;
    impl ToHandler for Count {
        type BaseHandler = CompStruct;
    }
    #[allow(non_snake_case)]
    pub struct CounterHolder {
        comp_struct: CompStruct,
        widget: ::gui::gui_widget::Button<Count, CompStruct, ::gui::gui_widget::Text>,
        Counter_Text_text: FluentArgs<'static>,
    }
    #[automatically_derived]
    impl ToComponent for CompStruct {
        type Component = CounterHolder;
        fn to_component_holder(self) -> Self::Component {
            CounterHolder {
                widget: ::gui::gui_widget::Button::new(
                    ::gui::gui_core::Colour::rgba8(255u8, 255u8, 255u8, 255u8),
                    ::gui::gui_core::Colour::rgba8(241u8, 243u8, 245u8, 255u8),
                    ::gui::gui_core::Colour::rgba8(248u8, 249u8, 250u8, 255u8),
                    ::gui::gui_core::Colour::rgba8(248u8, 249u8, 250u8, 255u8),
                    ::gui::gui_core::Colour::rgba8(206u8, 212u8, 218u8, 255u8),
                    false,
                    ::gui::gui_widget::Text::new(
                        String::new(),
                        ::gui::gui_core::Colour::rgba8(33u8, 37u8, 41u8, 255u8),
                        14f32,
                    ),
                ),
                comp_struct: self,
                Counter_Text_text: FluentArgs::new(),
            }
        }
    }
    #[automatically_derived]
    impl Component for CounterHolder {
        fn render(&mut self, scene: SceneBuilder, fcx: &mut FontContext) {
            self.widget.render(scene, fcx);
        }
        fn update_vars(&mut self, force_update: bool) {
            let mut text = false;
            if force_update || <CompStruct as Update<count>>::is_updated(&self.comp_struct) {
                <CompStruct as Update<count>>::reset(&mut self.comp_struct);
                let value = <CompStruct as Update<count>>::value(&self.comp_struct);
                let widget = &mut self.widget;
                let widget = &mut self.widget.get_widget();
                widget.set_size(value);
                text = true;
                self.Counter_Text_text.set("count", value);
            }
            if force_update || <CompStruct as Update<disabled>>::is_updated(&self.comp_struct) {
                <CompStruct as Update<disabled>>::reset(&mut self.comp_struct);
                let value = <CompStruct as Update<disabled>>::value(&self.comp_struct);
                let widget = &mut self.widget;
                widget.set_disabled(value);
                let widget = &mut self.widget.get_widget();
            }
            if force_update || text {
                let value = get_bundle_message("Counter-Text-text", Some(&self.Counter_Text_text));
                let widget = &mut self.widget.get_widget();
                widget.set_text(value);
            }
        }
        fn pointer_down(&mut self, event: &PointerEvent, window: &WindowHandle) {
            self.widget
                .pointer_down(event, window, &mut self.comp_struct);
        }
        fn pointer_up(&mut self, event: &PointerEvent, window: &WindowHandle) {
            self.widget.pointer_up(event, window, &mut self.comp_struct);
        }
        fn pointer_move(&mut self, event: &PointerEvent, window: &WindowHandle) {
            self.widget
                .pointer_move(event, window, &mut self.comp_struct);
        }
    }
}
