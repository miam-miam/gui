use gui::gui_widget::button::ButtonHandler;
use gui::{ToComponent, Update, Updateable};

#[derive(ToComponent, Default)]
struct Counter {
    count: Updateable<f32>,
}

impl Update<gen::disabled> for Counter {
    fn is_updated(&self) -> bool {
        self.count.is_updated()
    }

    fn value(&self) -> bool {
        self.count.value() > 15.0
    }
}

impl ButtonHandler<gen::Count> for Counter {
    fn on_press(&mut self) {
        *self.count.invalidate() += 1.0;
    }
}
fn main() {
    gui::run(Counter::default())
}
