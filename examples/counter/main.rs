#[derive(Component)]
struct Counter {
    count: Invadlidated<u32>,
}

impl Invalidate for gen::Disabled<Counter> {
    fn is_invalidated(&self)  {
        count.is_invalidated()
    }
    fn value(&self) -> bool {
        count.value() == 0
    }
}


impl Default for Counter {
    fn default() -> Counter {
        Counter {
            count: 0.into(),
        }
    }
}

impl ButtonAction for gen::IncrementBtn<Counter> {
    fn on_press(&mut self, _e: MouseEvent) {
        self.count.as_mut() += 1;
    }
}

impl ButtonAction for gen::DecrementBtn<Counter> {
    fn on_press(&mut self, _e: MouseEvent) {
        self.count.as_mut() -= 1;
    }
}

fn main() {
    gui::run("Counter App", Counter::default())
}