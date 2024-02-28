mod component;
mod fluent;
mod widget;
mod widget_set;

use anyhow::{anyhow, bail, Context};
use gui_core::parse::{GUIDeclaration, WidgetDeclaration};
use gui_core::widget::{AsAny, WidgetBuilder};
use itertools::Itertools;
use std::any::TypeId;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::ops::Not;
use std::path::Path;

extern crate gui_widget;

pub fn build<P: AsRef<Path>>(path: P) {
    if let Err(e) = build_path(path.as_ref()) {
        println!("{e:#}");
        std::process::exit(1);
    }
}

fn build_path(path: &Path) -> anyhow::Result<()> {
    let file = File::open(path).context("Failed to open GUI configuration file")?;
    let out_dir_env =
        env::var_os("OUT_DIR").ok_or_else(|| anyhow!("could not find OUT_DIR env"))?;
    let out_dir = Path::new(&out_dir_env);
    let mut ser: GUIDeclaration =
        serde_yaml::from_reader(file).context("Failed to parse file GUI configuration file")?;

    combine_styles(&mut ser).context("Failed to combine styles")?;

    for component in ser.components.iter_mut() {
        component::create_component(out_dir, component)
            .with_context(|| format!("Failed to create component {}", component.name.as_str()))?;
    }

    println!("cargo:rerun-if-changed={}", path.display());
    println!("cargo:rerun-if-changed=build.rs");
    add_info_to_env(&ser);

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
        combine_style(&mut c.child, &styles, &static_gui.styles[..])
    }

    Ok(())
}

fn combine_style(
    widget: &mut WidgetDeclaration,
    style_map: &HashMap<TypeId, usize>,
    styles: &[Box<dyn WidgetBuilder>],
) {
    if let Some(i) = style_map.get(&(widget.widget.as_any().type_id())) {
        widget.widget.combine(styles[*i].as_ref())
    }

    for child in widget.widget.get_widgets().into_iter().flatten() {
        combine_style(child, style_map, styles);
    }
}

fn add_info_to_env(static_gui: &GUIDeclaration) {
    let components = static_gui
        .components
        .iter()
        .map(|c| c.name.as_str())
        .format(",");
    println!("cargo:rustc-env=GUI_COMPONENTS={components}");
    for component in &static_gui.components {
        let state_name = component
            .states
            .is_empty()
            .not()
            .then_some("state")
            .into_iter();
        let variables = component
            .variables
            .iter()
            .map(|v| v.get_name().as_str())
            .chain(state_name)
            .format(",");
        println!(
            "cargo:rustc-env=GUI_COMPONENT_{}={variables}",
            component.name.as_str()
        );
    }
}
