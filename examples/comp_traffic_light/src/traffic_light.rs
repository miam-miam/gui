use gui::gui_core::OnMessage;
use gui::{ToComponent, Updateable};

#[derive(ToComponent, Default)]
pub struct TrafficLight {
    state: Updateable<gen::State>,
}

pub enum Message {
    Next,
}

impl OnMessage for TrafficLight {
    type Message = Message;

    fn on_message(&mut self, message: Self::Message) {
        match message {
            Message::Next => {
                use gen::State;
                let next = match self.state.value() {
                    State::Red => State::RedYellow,
                    State::RedYellow => State::Green,
                    State::Green => State::Yellow,
                    State::Yellow => State::Red,
                };
                *self.state.invalidate() = next;
            }
        }
    }
}
