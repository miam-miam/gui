use std::path::Path;

fn main() {
    let p = Path::new(file!());
    let simple = p.parent().unwrap().parent().unwrap().join("simple.yaml");
    let out_dir = p.parent().unwrap().parent().unwrap().join("OUT_DIR");
    std::env::set_var("OUT_DIR", out_dir);
    gui_build::build(simple);
}
