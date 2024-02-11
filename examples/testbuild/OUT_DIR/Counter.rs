#[allow(clippy::suspicious_else_formatting)]
mod gen {
    use super::__private_CompStruct as CompStruct;
    use gui::gui_core::glazier::{PointerEvent, WindowHandle};
    use gui::gui_core::parley::font::FontContext;
    use gui::gui_core::vello::SceneBuilder;
    use gui::gui_core::widget::Widget;
    use gui::gui_core::{
        Component, LayoutConstraints, Size, Point, ToComponent, ToHandler, Update,
        Variable,
    };
    enum WidgetSet0 {
        W0(::gui::gui_widget::Text),
        W1(::gui::gui_widget::Button<Count, CompStruct, ::gui::gui_widget::Text>),
    }
    impl WidgetSet0 {
        pub fn w0(&mut self) -> &mut ::gui::gui_widget::Text {
            if let WidgetSet0::W0(val) = self {
                val
            } else {
                panic!("Incorrect wrapped type.")
            }
        }
        pub fn w1(
            &mut self,
        ) -> &mut ::gui::gui_widget::Button<Count, CompStruct, ::gui::gui_widget::Text> {
            if let WidgetSet0::W1(val) = self {
                val
            } else {
                panic!("Incorrect wrapped type.")
            }
        }
    }
    impl Widget<CompStruct> for WidgetSet0 {
        fn render(&mut self, scene: &mut SceneBuilder, fcx: &mut FontContext) {
            match self {
                WidgetSet0::W0(w) => {
                    <::gui::gui_widget::Text as Widget<
                        CompStruct,
                    >>::render(w, scene, fcx)
                }
                WidgetSet0::W1(w) => {
                    <::gui::gui_widget::Button<
                        Count,
                        CompStruct,
                        ::gui::gui_widget::Text,
                    > as Widget<CompStruct>>::render(w, scene, fcx)
                }
            }
        }
        fn resize(
            &mut self,
            constraints: LayoutConstraints,
            fcx: &mut FontContext,
        ) -> Size {
            match self {
                WidgetSet0::W0(w) => {
                    <::gui::gui_widget::Text as Widget<
                        CompStruct,
                    >>::resize(w, constraints, fcx)
                }
                WidgetSet0::W1(w) => {
                    <::gui::gui_widget::Button<
                        Count,
                        CompStruct,
                        ::gui::gui_widget::Text,
                    > as Widget<CompStruct>>::resize(w, constraints, fcx)
                }
            }
        }
        fn pointer_down(
            &mut self,
            local_pos: Point,
            event: &PointerEvent,
            window: &WindowHandle,
            handler: &mut CompStruct,
        ) {
            match self {
                WidgetSet0::W0(w) => {
                    <::gui::gui_widget::Text as Widget<
                        CompStruct,
                    >>::pointer_down(w, local_pos, event, window, handler)
                }
                WidgetSet0::W1(w) => {
                    <::gui::gui_widget::Button<
                        Count,
                        CompStruct,
                        ::gui::gui_widget::Text,
                    > as Widget<
                        CompStruct,
                    >>::pointer_down(w, local_pos, event, window, handler)
                }
            }
        }
        fn pointer_up(
            &mut self,
            local_pos: Point,
            event: &PointerEvent,
            window: &WindowHandle,
            handler: &mut CompStruct,
        ) {
            match self {
                WidgetSet0::W0(w) => {
                    <::gui::gui_widget::Text as Widget<
                        CompStruct,
                    >>::pointer_up(w, local_pos, event, window, handler)
                }
                WidgetSet0::W1(w) => {
                    <::gui::gui_widget::Button<
                        Count,
                        CompStruct,
                        ::gui::gui_widget::Text,
                    > as Widget<
                        CompStruct,
                    >>::pointer_up(w, local_pos, event, window, handler)
                }
            }
        }
        fn pointer_move(
            &mut self,
            local_pos: Point,
            event: &PointerEvent,
            window: &WindowHandle,
            handler: &mut CompStruct,
        ) {
            match self {
                WidgetSet0::W0(w) => {
                    <::gui::gui_widget::Text as Widget<
                        CompStruct,
                    >>::pointer_move(w, local_pos, event, window, handler)
                }
                WidgetSet0::W1(w) => {
                    <::gui::gui_widget::Button<
                        Count,
                        CompStruct,
                        ::gui::gui_widget::Text,
                    > as Widget<
                        CompStruct,
                    >>::pointer_move(w, local_pos, event, window, handler)
                }
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
        const FTL_STRING: &str = include_str!(concat!(env!("OUT_DIR"), "/Counter.ftl"));
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
    pub(crate) struct Count;
    impl ToHandler for Count {
        type BaseHandler = CompStruct;
    }
    #[allow(non_snake_case)]
    pub struct CounterHolder {
        comp_struct: CompStruct,
        widget: ::gui::gui_widget::HBox<CompStruct, WidgetSet0>,
    }
    #[automatically_derived]
    impl ToComponent for CompStruct {
        type Component = CounterHolder;
        fn to_component_holder(self) -> Self::Component {
            CounterHolder {
                widget: ::gui::gui_widget::HBox::new(
                    10f32,
                    vec![
                        WidgetSet0::W0(::gui::gui_widget::Text::new(String::new(),
                        ::gui::gui_core::Colour::rgba8(33u8, 37u8, 41u8, 255u8), 24f32)),
                        WidgetSet0::W1(::gui::gui_widget::Button::new(::gui::gui_core::Colour::rgba8(255u8,
                        255u8, 255u8, 255u8), ::gui::gui_core::Colour::rgba8(241u8,
                        243u8, 245u8, 255u8), ::gui::gui_core::Colour::rgba8(248u8,
                        249u8, 250u8, 255u8), ::gui::gui_core::Colour::rgba8(248u8,
                        249u8, 250u8, 255u8), ::gui::gui_core::Colour::rgba8(206u8,
                        212u8, 218u8, 255u8), false,
                        ::gui::gui_widget::Text::new(String::new(),
                        ::gui::gui_core::Colour::rgba8(33u8, 37u8, 41u8, 255u8), 24f32)))
                    ],
                ),
                comp_struct: self,
            }
        }
    }
    #[automatically_derived]
    impl Component for CounterHolder {
        fn render(&mut self, mut scene: SceneBuilder, fcx: &mut FontContext) {
            self.widget.render(&mut scene, fcx);
        }
        fn update_vars(&mut self, force_update: bool) {
            if force_update {
                let value = get_bundle_message("Counter-Other-text", None);
                let widget = &mut self.widget.widgets(0usize).w0();
                widget.set_text(value);
            }
            if force_update {
                let value = get_bundle_message("Counter-Text-text", None);
                let widget = &mut self.widget.widgets(1usize).w1().get_widget();
                widget.set_text(value);
            }
        }
        fn resize(
            &mut self,
            constraints: LayoutConstraints,
            fcx: &mut FontContext,
        ) -> Size {
            self.widget.resize(constraints, fcx)
        }
        fn pointer_down(
            &mut self,
            local_pos: Point,
            event: &PointerEvent,
            window: &WindowHandle,
        ) {
            self.widget.pointer_down(local_pos, event, window, &mut self.comp_struct);
        }
        fn pointer_up(
            &mut self,
            local_pos: Point,
            event: &PointerEvent,
            window: &WindowHandle,
        ) {
            self.widget.pointer_up(local_pos, event, window, &mut self.comp_struct);
        }
        fn pointer_move(
            &mut self,
            local_pos: Point,
            event: &PointerEvent,
            window: &WindowHandle,
        ) {
            self.widget.pointer_move(local_pos, event, window, &mut self.comp_struct);
        }
    }
}
