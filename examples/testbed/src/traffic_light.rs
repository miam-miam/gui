use gui::gui_core::{Colour, OnMessage};
use gui::gui_widget::button::ButtonHandler;
use gui::{ToComponent, Updateable};

#[derive(ToComponent, Default)]
pub struct TrafficLight {
    state: Updateable<gen::State>,
    hover_colour: Updateable<Colour>,
    count: Updateable<u32>,
}

pub enum Message {
    Next,
}

impl ButtonHandler<gen::Switch> for TrafficLight {
    fn on_press(&mut self) {
        self.on_message(Message::Next)
    }
}

impl OnMessage for TrafficLight {
    type Message = Message;

    fn on_message(&mut self, message: Self::Message) {
        match message {
            Message::Next => {
                use gen::State;
                let next = match self.state.value() {
                    State::Green => State::Yellow,
                    State::Yellow => State::Red,
                    State::Red => State::Green,
                };
                *self.state.invalidate() = next;
            }
        }
    }
}
