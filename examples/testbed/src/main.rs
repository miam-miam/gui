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
