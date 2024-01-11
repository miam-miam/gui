fn main() {
    gui::run(include!(concat!(env!("OUT_DIR"), "/widget.rs")))
}
