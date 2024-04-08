use crate::traffic_light::{Message, TrafficLight};
use gui::gui_widget::button::ButtonHandler;
use gui::CompHolder;
use gui::{type_registry, ToComponent};

mod traffic_light;

#[derive(ToComponent, Default)]
struct Intersection {
    nw_light: CompHolder<TrafficLight>,
    ne_light: CompHolder<TrafficLight>,
    se_light: CompHolder<TrafficLight>,
    sw_light: CompHolder<TrafficLight>,
    enabled: EnabledLights,
}

impl Intersection {
    fn increment_lights(&mut self) {
        match self.enabled {
            EnabledLights::None => {}
            EnabledLights::Horizontal(_) => {
                self.ne_light.send_message(Message::Next);
                self.sw_light.send_message(Message::Next);
            }
            EnabledLights::Vertical(_) => {
                self.nw_light.send_message(Message::Next);
                self.se_light.send_message(Message::Next);
            }
        }
    }
}

#[derive(Default, Copy, Clone)]
enum EnabledLights {
    // All lights first start stopped.
    #[default]
    None,
    Horizontal(u8),
    Vertical(u8),
}

impl ButtonHandler<gen::Button> for Intersection {
    fn on_press(&mut self) {
        self.enabled = match self.enabled {
            EnabledLights::None | EnabledLights::Vertical(3) => EnabledLights::Horizontal(0),
            EnabledLights::Horizontal(3) => EnabledLights::Vertical(0),
            EnabledLights::Vertical(count) => EnabledLights::Vertical(count + 1),
            EnabledLights::Horizontal(count) => EnabledLights::Horizontal(count + 1),
        };
        self.increment_lights()
    }
}

type_registry!();

fn main() {
    gui::run(Intersection::default())
}

#[cfg(test)]
mod test {
    use crate::{EnabledLights, Intersection};
    use gui::{assert_screenshot, PointerButton, TestHarness};

    #[test]
    fn correct_state_transitions() {
        let mut harness = TestHarness::new(Intersection::default(), (800.0, 800.0));
        let next_id = harness.get_id("Button").unwrap();
        assert_screenshot!(harness, "start_all_off");

        for _ in 0..4 {
            harness.simulate_pointer_down_up(PointerButton::Primary, Some(next_id));
            assert!(matches!(
                harness.get_component().enabled,
                EnabledLights::Horizontal(_)
            ));
            assert_screenshot!(harness, "horizontal_starts");
        }

        for _ in 0..4 {
            harness.simulate_pointer_down_up(PointerButton::Primary, Some(next_id));
            assert!(matches!(
                harness.get_component().enabled,
                EnabledLights::Vertical(_)
            ));
            assert_screenshot!(harness, "vertical_next");
        }

        for _ in 0..4 {
            harness.simulate_pointer_down_up(PointerButton::Primary, Some(next_id));
            assert!(matches!(
                harness.get_component().enabled,
                EnabledLights::Horizontal(_)
            ));
            assert_screenshot!(harness, "horizontal_restarts");
        }
    }
}
