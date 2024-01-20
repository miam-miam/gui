use gui::gui_widget::button::ButtonHandler;
use gui::{ToComponent, Updateable};

#[derive(ToComponent, Default)]
struct Counter {
    name: Updateable<String>,
}

//
// use gui::gui_widget::button::ButtonHandler;
// use Counter as __private_CompStruct;

// impl ::gui::Update<gen::name> for Counter {
//     fn is_updated(&self) -> bool {
//         self.name.is_updated()
//     }
//     fn value(&self) -> <gen::name as ::gui::gui_core::Variable>::VarType {
//         self.name.value()
//     }
// }

impl ButtonHandler<gen::Count> for Counter {
    fn on_press(&self) {
        println!("Hiya!");
    }
}
//
// #[allow(clippy::suspicious_else_formatting)]
// mod gen {
//     use super::__private_CompStruct as CompStruct;
//     use gui::gui_core::glazier::PointerEvent;
//     use gui::gui_core::parley::font::FontContext;
//     use gui::gui_core::vello::peniko::Color;
//     use gui::gui_core::vello::SceneBuilder;
//     use gui::gui_core::widget::Widget;
//     use gui::gui_core::{Colour, Component, ToComponent, Update, Variable};
//     use gui::gui_widget::button::ToHandler;
//     use gui::gui_widget::Text;
//
//     #[allow(non_camel_case_types)]
//     pub(crate) struct name;
//     impl Variable for name {
//         type VarType = String;
//     }
//
//     // /*
//     pub(crate) struct Count;
//
//     impl ToHandler for Count {
//         type BaseHandler = CompStruct;
//     }
//
//     // */
//     #[allow(non_snake_case)]
//     pub struct CounterHolder {
//         comp_struct: CompStruct,
//         widget: ::gui::gui_widget::Button<Count, CompStruct, Text>, // modify type.
//     }
//     #[automatically_derived]
//     impl ToComponent for CompStruct {
//         type Component = CounterHolder;
//         fn to_component_holder(self) -> Self::Component {
//             CounterHolder {
//                 widget: ::gui::gui_widget::Button::new(
//                     ::gui::gui_core::Colour::rgba8(72u8, 126u8, 176u8, 255u8),
//                     false,
//                     Text::new(String::from("Hiya"), Colour(Color::YELLOW), 25.0),
//                 ),
//                 comp_struct: self,
//             }
//         }
//     }
//     #[automatically_derived]
//     impl Component for CounterHolder {
//         fn render(&mut self, scene: SceneBuilder, fcx: &mut FontContext) {
//             self.widget.render(scene, fcx);
//         }
//         fn update_vars(&mut self, force_update: bool) {
//             if force_update || <CompStruct as Update<name>>::is_updated(&self.comp_struct) {
//                 let value = <CompStruct as Update<name>>::value(&self.comp_struct);
//                 let widget = &mut self.widget;
//             }
//         }
//
//         // /*
//
//         fn pointer_up(&mut self, event: &PointerEvent) {
//             self.widget.on_press(&mut self.comp_struct)
//         }
//
//         // */
//     }
// }

fn main() {
    gui::run(Counter::default())
}
