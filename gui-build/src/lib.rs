use anyhow::{anyhow, bail};
use gui_core::parse::GUIDeclaration;
use gui_core::widget::AsAny;
use proc_macro2::TokenStream;
use std::any::{Any, TypeId};
use std::collections::HashMap;
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
    let mut ser: GUIDeclaration = serde_yaml::from_reader(file)?;
    let mut stream = TokenStream::new();

    combine_styles(&mut ser)?;

    ser.components[0].child.widget.create_widget(&mut stream);

    let path = Path::new(&out_dir).join("widget.rs");
    fs::write(path, format!("{}", stream))?;
    Ok(())
}

fn combine_styles(static_gui: &mut GUIDeclaration) -> anyhow::Result<()> {
    let mut styles: HashMap<TypeId, usize> = HashMap::new();

    for (t, i) in static_gui
        .styles
        .iter()
        .enumerate()
        .map(|(i, s)| (s.as_any().type_id(), i))
    {
        if styles.insert(t, i).is_some() {
            bail!(
                "Found multiple styles for widget {}",
                static_gui.styles[i].name()
            )
        }
    }

    for c in &mut static_gui.components {
        let widget = &c.child.widget;
        if let Some(i) = styles.get(&(widget.as_any().type_id())) {
            c.child.widget.combine(static_gui.styles[*i].as_ref())
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {}
