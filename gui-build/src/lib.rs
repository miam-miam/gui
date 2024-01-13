mod component;

use anyhow::{anyhow, bail};
use gui_core::parse::{ComponentDeclaration, GUIDeclaration};
use gui_core::widget::AsAny;
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

    for component in ser.components.iter_mut() {
        let bundle = create_bundle(component)?;
        let ftl_path = path.join(format!("{}.ftl", component.name));
        fs::write(ftl_path, bundle)?;
        component::create_component(path, component)?;
    }

    Ok(())
}

fn create_bundle(component: &ComponentDeclaration) -> anyhow::Result<String> {
    let mut bundle = String::new();
    for (property_name, fluent) in component.child.widget.get_fluents() {
        let fluent_name = format!(
            "{}-{}-{}",
            component.name,
            component
                .child
                .name
                .as_ref()
                .map_or_else(|| component.child.widget.name(), |s| s.as_str()),
            property_name
        );
        bundle = bundle + &format!("{fluent_name} = {}", fluent.text);
    }
    Ok(bundle)
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
