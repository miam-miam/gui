mod component;
mod fluent;
mod widget;

use anyhow::{anyhow, bail};
use gui_core::parse::{ComponentDeclaration, GUIDeclaration};
use gui_core::widget::AsAny;
use itertools::Itertools;
use std::any::TypeId;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::{env, fs};

extern crate gui_widget;

pub fn build<P: AsRef<Path>>(path: P) {
    build_path(path.as_ref()).unwrap()
}

fn build_path(path: &Path) -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed={}", path.display());
    println!("cargo:rerun-if-changed=build.rs");

    let file = File::open(path)?;
    let out_dir = env::var_os("OUT_DIR").ok_or_else(|| anyhow!("could not find OUT_DIR env"))?;
    let path = Path::new(&out_dir);
    let mut ser: GUIDeclaration = serde_yaml::from_reader(file)?;

    combine_styles(&mut ser)?;

    add_info_to_env(&ser);

    for component in ser.components.iter_mut() {
        component::create_component(path, component)?;
    }

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

fn add_info_to_env(static_gui: &GUIDeclaration) {
    let components = static_gui.components.iter().map(|c| &c.name).format(",");
    println!("cargo:rustc-env=GUI_COMPONENTS={components}");
    for component in &static_gui.components {
        let variables = component.variables.iter().map(|v| v.get_name()).format(",");
        println!(
            "cargo:rustc-env=GUI_COMPONENT_{}={variables}",
            component.name
        );
    }
}

#[cfg(test)]
mod tests {}
