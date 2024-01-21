use crate::gen::count;
use gui::gui_widget::button::ButtonHandler;
use gui::{ToComponent, Updateable};

#[derive(ToComponent)]
struct Counter {
    count: Updateable<f32>,
}

impl Default for Counter {
    fn default() -> Self {
        Self {
            count: Updateable::new(12.0),
        }
    }
}

impl ButtonHandler<gen::Count> for Counter {
    fn on_press(&mut self) {
        *self.count.invalidate() += 5.0;
    }
}
fn main() {
    gui::run(Counter::default())
}
