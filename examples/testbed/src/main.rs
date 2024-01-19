use gui::{ToComponent, Updateable};

#[derive(ToComponent, Default)]
struct Counter {
    name: Updateable<String>,
}

fn main() {
    gui::run(Counter::default())
}
