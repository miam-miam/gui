use gui::gui_widget::button::ButtonHandler;
use gui::{ToComponent, Update, Updateable};

#[derive(ToComponent, Default)]
struct Counter {
    count: Updateable<u32>,
}

impl Update<gen::disabled_decrement> for Counter {
    fn is_updated(&self) -> bool {
        self.count.is_updated()
    }
    fn value(&self) -> bool {
        self.count.value() == 0
    }
}

impl ButtonHandler<gen::IncrementBtn> for Counter {
    fn on_press(&mut self) {
        *self.count.invalidate() += 1;
    }
}

impl ButtonHandler<gen::DecrementBtn> for Counter {
    fn on_press(&mut self) {
        *self.count.invalidate() -= 1;
    }
}

fn main() {
    gui::run(Counter::default())
}

#[cfg(test)]
mod test {
    use crate::Counter;
    use gui::{assert_screenshot, PointerButton, TestHarness};

    #[test]
    fn test() {
        let mut harness = TestHarness::new(Counter::default());
        assert_screenshot!(harness, "valid_start_state");
        harness.set_size((1024.0, 1024.0));
        assert_screenshot!(harness, "larger_resize");
        harness.set_size((512.0, 512.0));
        let incr_btn = harness.get_id("IncrementBtn").unwrap();
        let decr_btn = harness.get_id("DecrementBtn").unwrap();

        harness.simulate_pointer_down_up(PointerButton::Primary, Some(decr_btn));
        assert_screenshot!(harness, "decrement_btn_cannot_be_pressed");

        harness.simulate_pointer_move(incr_btn, None);
        assert_screenshot!(harness, "increment_btn_hovered");

        harness.simulate_pointer_down(PointerButton::Primary, None);
        assert_screenshot!(harness, "increment_btn_pressed");

        harness.simulate_pointer_up(PointerButton::Primary, None);
        assert_screenshot!(harness, "counter_incremented");
        assert_eq!(*harness.get_component().count.get_value(), 1);

        harness.simulate_pointer_down_up(PointerButton::Primary, Some(decr_btn));
        assert_screenshot!(harness, "decrement_btn_pressed");
        assert_eq!(*harness.get_component().count.get_value(), 0);

        harness.simulate_pointer_down_up(PointerButton::Primary, Some(incr_btn));
        harness.simulate_pointer_down(PointerButton::Primary, Some(incr_btn));
        harness.simulate_pointer_move(decr_btn, None);
        assert_screenshot!(harness, "decrement_btn_is_not_hovered");

        assert_eq!(*harness.get_component().count.get_value(), 1);
        harness.simulate_pointer_up(PointerButton::Primary, None);
        assert_screenshot!(harness, "count_not_incremented");
        assert_eq!(*harness.get_component().count.get_value(), 1);
    }
}
