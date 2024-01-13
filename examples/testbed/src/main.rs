use gui::gui_core::Update;

struct Counter;
use Counter as __private_CompStruct;
include!(concat!(env!("OUT_DIR"), "/Counter.rs"));

impl Update<gen::name> for Counter {
    fn is_updated(&self) -> bool {
        false
    }
    fn value(&self) -> String {
        String::from("World")
    }
}

fn main() {
    gui::run(Counter)
}
