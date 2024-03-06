use crate::gen::State;
use gui::gui_core::Colour;
use gui::gui_widget::button::ButtonHandler;
use gui::{ToComponent, Update, Updateable};

#[derive(ToComponent, Default)]
struct TrafficLight {
    state: Updateable<State>,
    count: Updateable<u32>,
}

impl Update<gen::light_colour> for TrafficLight {
    fn is_updated(&self) -> bool {
        self.state.is_updated()
    }

    fn value(&self) -> Colour {
        match self.state.value() {
            State::Green => Colour::rgba8(170, 255, 128, 255),
            State::Yellow => Colour::rgba8(204, 204, 0, 255),
            State::Red => Colour::rgba8(254, 64, 25, 255),
        }
    }
}

impl ButtonHandler<gen::Switch> for TrafficLight {
    fn on_press(&mut self) {
        let next = match self.state.value() {
            State::Green => State::Yellow,
            State::Yellow => State::Red,
            State::Red => State::Green,
        };
        *self.state.invalidate() = next;
        if next == State::Red {
            *self.count.invalidate() += 1;
        }
    }
}

fn main() {
    gui::run(TrafficLight::default())
}

#[cfg(test)]
mod test {
    use crate::TrafficLight;
    use gui::{assert_screenshot, PointerButton, TestHarness};

    #[test]
    fn test() {
        let mut harness = TestHarness::new(TrafficLight::default(), (250.0, 250.0));
        let button = harness.get_id("Switch").unwrap();

        assert_screenshot!(harness, "start_green");

        harness.simulate_pointer_down_up(PointerButton::Primary, Some(button));
        assert_screenshot!(harness, "goes_to_yellow");

        harness.simulate_pointer_down_up(PointerButton::Primary, Some(button));
        assert_screenshot!(harness, "goes_to_red");

        for i in 2..6 {
            harness.simulate_pointer_down_up(PointerButton::Primary, Some(button));
            harness.simulate_pointer_down_up(PointerButton::Primary, Some(button));
            harness.simulate_pointer_down_up(PointerButton::Primary, Some(button));

            assert_eq!(*harness.get_component().count.get_value(), i)
        }

        assert_screenshot!(harness, "count_has_incremented");
    }
}
