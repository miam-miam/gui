#[derive(Component)]
struct Counter {
    count: Updateable<u32>,
}

impl Update for gen::disabled_decrement {
    fn is_updated(&self) {
        count.is_updated()
    }
    fn value(&self) -> bool {
        count.value() == 0
    }
}

impl Default for Counter {
    fn default() -> Counter {
        Counter { count: 0.into() }
    }
}

impl ButtonAction for gen::IncrementBtn {
    fn on_press(&mut self, _e: MouseEvent) {
        self.count.as_mut() += 1;
    }
}

impl ButtonAction for gen::DecrementBtn {
    fn on_press(&mut self, _e: MouseEvent) {
        self.count.as_mut() -= 1;
    }
}

fn main() {
    gui::run("Counter App", Counter::default())
}
