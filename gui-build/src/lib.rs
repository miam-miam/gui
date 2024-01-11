use anyhow::anyhow;
use gui_core::parse::GUIDeclaration;
use proc_macro2::TokenStream;
use std::fs::File;
use std::path::Path;
use std::{env, fs};

pub fn build<P: AsRef<Path>>(path: P) {
    build_path(path.as_ref()).unwrap()
}

fn build_path(path: &Path) -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed={}", path.display());
    println!("cargo:rerun-if-changed=build.rs");

    let file = File::open(path)?;
    let out_dir = env::var_os("OUT_DIR").ok_or_else(|| anyhow!("could not find OUT_DIR env"))?;
    let ser: GUIDeclaration = serde_yaml::from_reader(file)?;
    let mut stream = TokenStream::new();
    ser.components[0].child.widget.create_widget(&mut stream);
    let path = Path::new(&out_dir).join("widget.rs");
    fs::write(path, format!("{}", stream))?;
    dbg!(ser);
    Ok(())
}

#[cfg(test)]
mod tests {}
