use crate::traffic_light::TrafficLight;
use gui::CompHolder;
use gui::{type_registry, ToComponent};

mod traffic_light;

#[derive(ToComponent, Default)]
struct Holder {
    light: CompHolder<TrafficLight>,
}

type_registry!();

fn main() {
    gui::run(Holder::default())
}
