use anyhow::{anyhow, bail};
use fluent_bundle::{FluentBundle, FluentResource};
use gui_core::parse::{
    ComponentDeclaration, GUIDeclaration, NormalVariableDeclaration, VariableDeclaration,
};
use gui_core::widget::AsAny;
use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::any::TypeId;
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::path::Path;
use std::str::FromStr;
use std::{env, fs};
use unic_langid::LanguageIdentifier;

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
        create_component(path, bundle, component)?;
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

fn create_component(
    out_dir: &Path,
    bundle: String,
    component: &ComponentDeclaration,
) -> anyhow::Result<()> {
    let mut stream = TokenStream::new();
    component.child.widget.create_widget(&mut stream);

    let rs_path = Path::new(&out_dir).join(format!("{}.rs", component.name));
    let ftl_path = Path::new(&out_dir).join(format!("{}.ftl", component.name));

    fs::write(ftl_path, bundle)?;

    let variables: HashMap<&str, &NormalVariableDeclaration> = component
        .variables
        .iter()
        .filter_map(|v| match v {
            VariableDeclaration::Normal(n) => Some((n.name.as_str(), n)),
            _ => None,
        })
        .collect();

    let fluents = component.child.widget.get_fluents();

    let var_to_update_fluent = fluents
        .iter()
        .flat_map(|(prop, fluent)| {
            fluent
                .vars
                .iter()
                .map(move |v| (v.as_str(), (prop, *fluent)))
        })
        .into_group_map();

    let widget_vars: HashMap<&str, Vec<&str>> = component
        .child
        .widget
        .get_vars()
        .iter()
        .map(|(prop, var)| (*var, *prop))
        .into_group_map();

    let if_update: TokenStream = variables
        .values()
        .map(|v| {
            let var_name = Ident::new(&v.name, Span::call_site());
            let mut stream = TokenStream::new();
            let widget = Ident::new("widget", Span::call_site());
            let value = Ident::new("value", Span::call_site());
            for prop in widget_vars
                .get(v.name.as_str())
                .expect("variable should exist")
            {
                component
                    .child
                    .widget
                    .on_property_update(prop, &widget, &value, &mut stream);
            }

            let string_var_name = &v.name;

            let stream2: TokenStream = var_to_update_fluent
                .get(v.name.as_str())
                .iter()
                .flat_map(|fluents| fluents.iter())
                .map(|(prop, _fluent)| {
                    let fluent_name = Ident::new(
                        &format!(
                            "{}_{}_{}",
                            component.name,
                            component
                                .child
                                .name
                                .as_ref()
                                .map_or_else(|| component.child.widget.name(), |s| s.as_str()),
                            prop
                        ),
                        Span::call_site(),
                    );
                    let prop = Ident::new(prop, Span::call_site());
                    quote! {
                        #prop = true;
                        self.#fluent_name.set(#string_var_name, #value);
                    }
                })
                .collect();
            quote! {
                if force_update || <Self as Update<#var_name>>::is_updated(self) {
                    let #value = <Self as Update<#var_name>>::value(self);
                    let #widget = &mut self.widget;
                    #stream
                    #stream2
                }
            }
        })
        .collect();

    let prop_set: TokenStream = fluents
        .iter()
        .map(|(prop, _fluent)| {
            let prop = Ident::new(prop, Span::call_site());
            quote! {
                let mut #prop = false;
            }
        })
        .collect();

    let prop_update: TokenStream = fluents
        .iter()
        .map(|(prop, fluent)| {
            let property = Ident::new(prop, Span::call_site());
            let mut stream = TokenStream::new();
            let widget = Ident::new("widget", Span::call_site());
            let value = Ident::new("value", Span::call_site());
            let fluent_name = &format!(
                "{}-{}-{}",
                component.name,
                component
                    .child
                    .name
                    .as_ref()
                    .map_or_else(|| component.child.widget.name(), |s| s.as_str()),
                prop
            );
            let fluent_arg = Ident::new(
                &format!(
                    "{}_{}_{}",
                    component.name,
                    component
                        .child
                        .name
                        .as_ref()
                        .map_or_else(|| component.child.widget.name(), |s| s.as_str()),
                    prop
                ),
                Span::call_site(),
            );
            let arg = if fluent.vars.is_empty() {
                quote! {None}
            } else {
                quote! {Some(self.#fluent_arg)}
            };
            component
                .child
                .widget
                .on_property_update(prop, &widget, &value, &mut stream);
            quote! {
                if force_update || #property {
                    let value = get_bundle_message(#fluent_name, #arg);
                    let #widget = &mut self.widget;
                    #stream
                }
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
        rs_path,
        format!(
            "{}",
            quote! {
                mod gen {
                    use gui::gui_core::widget::Widget;
                    use gui::gui_core::{Update, Variable, Component};
                    use gui::gui_core::vello::SceneBuilder;
                    use gui::gui_core::parley::font::FontContext;
                    use gui::{FluentBundle, FluentMessage, FluentArgs, FluentResource};
                    use std::borrow::Cow;
                    use super::CompStruct;

                    fn get_bundle_message<'a>(message: &'a str, args: Option<&'a FluentArgs<'_>>) -> Cow<'a, str> {
                        use std::sync::OnceLock;
                        use gui::langid;

                        static BUNDLE: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();
                        const FTL_STRING: &'static str = include_str!(concat!(env!("OUT_DIR"), "/Counter.ftl"));
                        let mut errors = vec![];
                        let bundle = BUNDLE.get_or_init(|| {
                            let mut bundle = FluentBundle::new_concurrent(vec![langid!("en-GB")]);
                            let resource = FluentResource::try_new(FTL_STRING.to_string())
                                .expect("FTL string is valid.");
                            bundle.add_resource(resource).expect("No identifiers are overlapping.");
                            bundle
                        });
                        let message = bundle.get_message(message).expect("Message exists.");
                        let pattern = message.value().expect("Value exists.");
                        bundle.format_pattern(&pattern, args, &mut errors)
                    }

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
                            #prop_set
                            #if_update
                            #prop_update
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
