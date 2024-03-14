use crate::traffic_light::TrafficLight;
use gui::gui_widget::button::ButtonHandler;
use gui::CompHolder;
use gui::{type_registry, ToComponent};

mod traffic_light;

#[derive(ToComponent, Default)]
struct Holder {
    light: CompHolder<TrafficLight>,
}

impl ButtonHandler<gen::Button> for Holder {
    fn on_press(&mut self) {
        self.light.send_message(traffic_light::Message::Next);
        println!("Send message");
    }
}

type_registry!();

fn main() {
    gui::run(Holder::default())
}
