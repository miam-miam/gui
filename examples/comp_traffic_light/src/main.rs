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
    }
}

type_registry!();

fn main() {
    gui::run(Holder::default())
}

#[cfg(test)]
mod test {
    use crate::Holder;
    use gui::{assert_screenshot, PointerButton, TestHarness};

    #[test]
    fn correct_state_transitions() {
        let mut harness = TestHarness::new(Holder::default(), (350.0, 800.0));
        let next_id = harness.get_id("Button").unwrap();
        assert_screenshot!(harness, "stop_state");
        harness.simulate_pointer_down_up(PointerButton::Primary, Some(next_id));
        assert_screenshot!(harness, "ready_state");
        harness.simulate_pointer_down_up(PointerButton::Primary, Some(next_id));
        assert_screenshot!(harness, "go_state");
        harness.simulate_pointer_down_up(PointerButton::Primary, Some(next_id));
        assert_screenshot!(harness, "yellow_state");
        harness.simulate_pointer_down_up(PointerButton::Primary, Some(next_id));
        assert_screenshot!(harness, "back_to_start");
    }
}
