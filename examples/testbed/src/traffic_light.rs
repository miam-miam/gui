use gui::gui_core::Colour;
use gui::gui_widget::button::ButtonHandler;
use gui::{ToComponent, Updateable};

#[derive(ToComponent, Default)]
pub struct TrafficLight {
    state: Updateable<gen::State>,
    hover_colour: Updateable<Colour>,
    count: Updateable<u32>,
}

impl ButtonHandler<gen::Switch> for TrafficLight {
    fn on_press(&mut self) {
        use gen::State;
        let next = match self.state.value() {
            State::Green => State::Yellow,
            State::Yellow => State::Red,
            State::Red => State::Green,
        };
        *self.state.invalidate() = next;
        if next == State::Red {
            *self.count.invalidate() += 1;
            *self.hover_colour.invalidate() =
                Colour::rgba8(255 - (self.count.value() * 10) as u8, 0, 0, 255)
        }
    }
}
