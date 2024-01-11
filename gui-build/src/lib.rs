use anyhow::{anyhow, bail};
use fluent_bundle::{FluentBundle, FluentResource};
use gui_core::parse::{ComponentDeclaration, GUIDeclaration, VariableDeclaration};
use gui_core::widget::AsAny;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::any::TypeId;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::str::FromStr;
use std::{env, fs};
use unic_langid::LanguageIdentifier;

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

    let lang_id: LanguageIdentifier = "en-GB".parse()?;
    let mut bundle = FluentBundle::<FluentResource>::new(vec![lang_id]);

    combine_styles(&mut ser)?;

    for component in ser.components.iter_mut() {
        create_component(path, component)?;
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

fn create_component(out_dir: &Path, component: &mut ComponentDeclaration) -> anyhow::Result<()> {
    let mut stream = TokenStream::new();
    component.child.widget.create_widget(&mut stream);

    let path = Path::new(&out_dir).join(format!("{}.rs", component.name));

    let update_vars: TokenStream = component
        .variables
        .iter()
        .filter_map(|v| {
            if let VariableDeclaration::Normal(n) = v {
                let mut stream = TokenStream::new();
                let var_name = Ident::new(&n.name, Span::call_site());
                let widget = Ident::new("widget", Span::call_site());
                let value = Ident::new("value", Span::call_site());
                component
                    .child
                    .widget
                    .on_var_update(&widget, &n.name, &value, &mut stream);
                Some(quote! {
                    if force_update || <Self as Update<#var_name>>::is_updated(self) {
                        let #value = <Self as Update<#var_name>>::value(self);
                        let #widget = &mut self.widget;
                        #stream
                    }
                })
            } else {
                None
            }
        })
        .collect();

    let struct_vars: TokenStream = component
        .variables
        .iter()
        .filter_map(|v| {
            if let VariableDeclaration::Normal(n) = v {
                let name = Ident::new(&n.name, Span::call_site());
                let var_type = TokenStream::from_str(&n.var_type).expect("a valid type");
                Some(quote! {
                    #[allow(non_camel_case_types)]
                    pub(crate) struct #name;

                    impl Variable for #name {
                        type VarType = #var_type;
                    }
                })
            } else {
                None
            }
        })
        .collect();

    fs::write(
        path,
        format!(
            "{}",
            quote! {
                mod gen {
                    use gui::gui_core::widget::Widget;
                    use gui::gui_core::{Update, Variable, Component};
                    use gui::gui_core::vello::SceneBuilder;
                    use gui::gui_core::parley::font::FontContext;
                    use super::CompStruct;

                    #struct_vars

                    impl Component for CompStruct {
                        fn new() -> Self where Self: Sized {
                            CompStruct {
                                widget: #stream
                            }
                        }

                        fn render(&mut self, scene: SceneBuilder, fcx: &mut FontContext) {
                            self.widget.render(scene, fcx);
                        }

                        fn update_vars(&mut self, force_update: bool) {
                            #update_vars
                        }
                    }
                }
            }
        ),
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {}
