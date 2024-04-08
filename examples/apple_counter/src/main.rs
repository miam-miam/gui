use gui::gui_widget::button::ButtonHandler;
use gui::Updateable;
use gui::{type_registry, ToComponent};

#[derive(ToComponent, Default)]
struct AppleCounter {
    apple_count: Updateable<u32>,
}

impl ButtonHandler<gen::AddApple> for AppleCounter {
    fn on_press(&mut self) {
        *self.apple_count.invalidate() += 1;
    }
}

impl ButtonHandler<gen::ResetApple> for AppleCounter {
    fn on_press(&mut self) {
        *self.apple_count.invalidate() = 0;
    }
}

type_registry!();

fn main() {
    gui::run(AppleCounter::default())
}

#[cfg(test)]
mod test {
    use crate::AppleCounter;
    use gui::{assert_screenshot, PointerButton, TestHarness};

    #[test]
    fn correct_resize() {
        let mut harness = TestHarness::new(AppleCounter::default(), (800.0, 400.0));
        assert_screenshot!(harness, "valid_start_state");
        harness.set_size((500.0, 400.0));
        assert_screenshot!(harness, "correct_text_wrap");
        harness.set_size((500.0, 1000.0));
        assert_screenshot!(harness, "height_unaffected")
    }

    #[test]
    fn correct_apple_counts() {
        let mut harness = TestHarness::new(AppleCounter::default(), (800.0, 400.0));
        let increment = harness.get_id("AddApple").unwrap();
        let reset = harness.get_id("ResetApple").unwrap();
        assert_screenshot!(harness, "no_apples");
        harness.simulate_pointer_down_up(PointerButton::Primary, Some(reset));
        assert_screenshot!(harness, "still_no_apples");
        harness.simulate_pointer_down_up(PointerButton::Primary, Some(increment));
        assert_screenshot!(harness, "one_apple");
        harness.simulate_pointer_down_up(PointerButton::Primary, Some(increment));
        assert_screenshot!(harness, "two_apples");
        harness.simulate_pointer_down_up(PointerButton::Primary, Some(reset));
        assert_screenshot!(harness, "correctly_reset");
        for count in 1..=10 {
            harness.simulate_pointer_down_up(PointerButton::Primary, Some(increment));
            assert_eq!(*harness.get_component().apple_count.get_value(), count);
        }
        assert_screenshot!(harness, "ten_apples")
    }
}
