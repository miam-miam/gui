use gui::gui_core::{Component, Update};
use rand::distributions::{Alphanumeric, DistString};

struct Counter {
    widget: ::gui_widget::Text,
}
use Counter as CompStruct;

include!(concat!(env!("OUT_DIR"), "/Counter.rs"));
impl Update<gen::test> for Counter {
    fn is_updated(&self) -> bool {
        true
    }
    fn value(&self) -> String {
        Alphanumeric.sample_string(&mut rand::thread_rng(), 16)
        // String::new()
    }
}

fn main() {
    gui::run(Counter::new())
}
