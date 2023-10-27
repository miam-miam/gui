#[derive(Component)]
struct Counter {
    count: Invadlidated<i32>,
    disabled_decrement: Invalidated<bool>,
}

impl Default for Counter {
    fn default() -> Counter {
        Counter {
            count: 0.into(),
            disabled_decrement: true.into(),
        }
    }
}

impl ButtonAction for gen::IncrementBtn<Counter> {
    fn on_press(&mut self, _e: MouseEvent) {
        self.count.as_mut() += 1;
        if self.count == 1 {
            self.disabled_decrement.as_mut() = false;
        }
    }
}

impl ButtonAction for gen::DecrementBtn<Counter> {
    fn on_press(&mut self, _e: MouseEvent) {
        self.count.as_mut() -= 1;
        if self.count == 0 {
            self.disabled_decrement.as_mut() = true;
        }
    }
}

fn main() {
    gui::run("Counter App", Counter::default())
}