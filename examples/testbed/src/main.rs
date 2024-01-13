use gui::gui_core::{Component, Update};

#[derive(Default)]
struct Counter;
use Counter as __private_CompStruct;
include!(concat!(env!("OUT_DIR"), "/Counter.rs"));

impl Update<gen::name> for Counter {
    fn is_updated(&self) -> bool {
        false
    }
    fn value(&self) -> String {
        String::from("Miam")
    }
}

fn main() {
    gui::run(gen::CounterHolder::new())
}
