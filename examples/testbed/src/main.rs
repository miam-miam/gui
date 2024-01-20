use gui::gui_widget::button::ButtonHandler;
use gui::{ToComponent, Updateable};

#[derive(ToComponent, Default)]
struct Counter {
    count: Updateable<u32>,
}

impl ButtonHandler<gen::Count> for Counter {
    fn on_press(&mut self) {
        *self.count.invalidate() += 1;
    }
}
fn main() {
    gui::run(Counter::default())
}
