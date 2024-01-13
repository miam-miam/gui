use gui_core::parse::fluent::Fluent;
use gui_core::parse::{ComponentDeclaration, NormalVariableDeclaration, VariableDeclaration};
use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;

struct FluentIdent<'a> {
    pub fluent: &'a Fluent,
    pub ident: Ident,
    pub name: String,
    pub property: &'static str,
    pub property_ident: Ident,
}

impl<'a> FluentIdent<'a> {
    pub fn new(
        property: &'static str,
        fluent: &'a Fluent,
        component: &ComponentDeclaration,
    ) -> Self {
        let widget_name = component
            .child
            .name
            .as_ref()
            .map_or_else(|| component.child.widget.name(), |s| s.as_str());
        Self {
            fluent,
            property,
            ident: Ident::new(
                &format!("{}_{widget_name}_{property}", component.name),
                Span::call_site(),
            ),
            name: format!("{}-{widget_name}-{property}", component.name),
            property_ident: Ident::new(property, Span::call_site()),
        }
    }
}

pub fn create_component(out_dir: &Path, component: &ComponentDeclaration) -> anyhow::Result<()> {
    let mut widget_init = TokenStream::new();
    component.child.widget.create_widget(&mut widget_init);

    let normal_variables: Vec<&NormalVariableDeclaration> = component
        .variables
        .iter()
        .filter_map(|v| match v {
            VariableDeclaration::Normal(n) => Some(n),
            _ => None,
        })
        .collect();

    let fluents = component
        .child
        .widget
        .get_fluents()
        .into_iter()
        .map(|(prop, fluent)| FluentIdent::new(prop, fluent, component))
        .collect_vec();

    let bundle_func = (!fluents.is_empty()).then(gen_bundle_function);

    let var_to_fluent = fluents
        .iter()
        .flat_map(|fluent| fluent.fluent.vars.iter().map(move |v| (v.as_str(), fluent)))
        .into_group_map();

    let widget_vars: HashMap<&str, Vec<&str>> = component
        .child
        .widget
        .get_vars()
        .iter()
        .map(|(prop, var)| (*var, *prop))
        .into_group_map();

    let if_update: TokenStream =
        gen_var_update(component, &normal_variables[..], var_to_fluent, widget_vars);

    let fluent_arg_idents: Vec<&Ident> = fluents.iter().map(|fluent| &fluent.ident).collect();

    let fluent_properties: Vec<&Ident> = fluents
        .iter()
        .map(|fluent| &fluent.property_ident)
        .collect();

    let prop_update: TokenStream = gen_fluent_update(component, &fluents[..]);

    let struct_vars: TokenStream = normal_variables
        .iter()
        .map(|n| {
            let name = Ident::new(&n.name, Span::call_site());
            let var_type = TokenStream::from_str(&n.var_type).expect("a valid type");
            quote! {
                #[allow(non_camel_case_types)]
                pub(crate) struct #name;

                impl Variable for #name {
                    type VarType = #var_type;
                }
            }
        })
        .collect();

    let component_holder = Ident::new(&format!("{}Holder", component.name), Span::call_site());

    let rs_path = Path::new(&out_dir).join(format!("{}.rs", component.name));

    let gen_module = quote! {
        #[allow(clippy::suspicious_else_formatting)]
        mod gen {
            use gui::gui_core::widget::Widget;
            use gui::gui_core::{Update, Variable, Component, ToComponent};
            use gui::gui_core::vello::SceneBuilder;
            use gui::gui_core::parley::font::FontContext;
            use super::__private_CompStruct as CompStruct;

            #bundle_func

            #struct_vars

            #[allow(non_snake_case)]
            pub struct #component_holder {
                comp_struct: CompStruct,
                widget: ::gui_widget::Text,
                #( #fluent_arg_idents: FluentArgs<'static> ),*
            }

            #[automatically_derived]
            impl ToComponent for CompStruct {
                type Component = #component_holder;

                fn to_component_holder(self) -> Self::Component {
                    #component_holder {
                        widget: #widget_init,
                        comp_struct: self,
                        #( #fluent_arg_idents: FluentArgs::new() ),*
                    }
                }
            }

            #[automatically_derived]
            impl Component for #component_holder {
                fn render(&mut self, scene: SceneBuilder, fcx: &mut FontContext) {
                    self.widget.render(scene, fcx);
                }

                fn update_vars(&mut self, force_update: bool) {
                    #( let mut #fluent_properties = false; )*
                    #if_update
                    #prop_update
                }
            }
        }
    };

    fs::write(rs_path, format!("{}", gen_module))?;

    Ok(())
}

fn gen_fluent_update(component: &ComponentDeclaration, fluents: &[FluentIdent]) -> TokenStream {
    fluents
        .iter()
        .map(|fluent| {
            let property_ident = &fluent.property_ident;
            let mut on_property_update = TokenStream::new();
            let widget = Ident::new("widget", Span::call_site());
            let value = Ident::new("value", Span::call_site());
            let fluent_name = &fluent.name;
            let fluent_arg = &fluent.ident;

            let arg = if fluent.fluent.vars.is_empty() {
                quote! {None}
            } else {
                quote! {Some(&self.#fluent_arg)}
            };
            component.child.widget.on_property_update(
                fluent.property,
                &widget,
                &value,
                &mut on_property_update,
            );
            quote! {
                if force_update || #property_ident {
                    let value = get_bundle_message(#fluent_name, #arg);
                    let #widget = &mut self.widget;
                    #on_property_update
                }
            }
        })
        .collect()
}

fn gen_var_update(
    component: &ComponentDeclaration,
    normal_variables: &[&NormalVariableDeclaration],
    var_to_fluent: HashMap<&str, Vec<&FluentIdent>>,
    widget_vars: HashMap<&str, Vec<&'static str>>,
) -> TokenStream {
    normal_variables
        .iter()
        .map(|v| {
            let var_name = Ident::new(&v.name, Span::call_site());
            let widget_ident = Ident::new("widget", Span::call_site());
            let value_ident = Ident::new("value", Span::call_site());
            let string_var_name = &v.name;
            let mut update_var_props = TokenStream::new();

            for prop in widget_vars
                .get(v.name.as_str())
                .into_iter()
                .flat_map(|props| props.iter())
            {
                component
                    .child
                    .widget
                    .on_property_update(prop, &widget_ident, &value_ident, &mut update_var_props);
            }

            let update_fluent_args = var_to_fluent
                .get(v.name.as_str())
                .into_iter()
                .flat_map(|fluents| fluents.iter())
                .map(|fluent| {
                    let fluent_ident = &fluent.ident;
                    let prop = Ident::new(fluent.property, Span::call_site());
                    quote! {
                        #prop = true;
                        self.#fluent_ident.set(#string_var_name, #value_ident);
                    }
                });

            quote! {
                if force_update || <CompStruct as Update<#var_name>>::is_updated(&self.comp_struct) {
                    let #value_ident = <CompStruct as Update<#var_name>>::value(&self.comp_struct);
                    let #widget_ident = &mut self.widget;
                    #update_var_props
                    #( #update_fluent_args )*
                }
            }
        })
        .collect()
}

fn gen_bundle_function() -> TokenStream {
    quote! {
        use gui::{FluentBundle, FluentArgs, FluentResource};
        use std::borrow::Cow;

        fn get_bundle_message<'a>(message: &'a str, args: Option<&'a FluentArgs<'_>>) -> Cow<'a, str> {
            use std::sync::OnceLock;
            use gui::langid;

            static BUNDLE: OnceLock<FluentBundle<FluentResource>> = OnceLock::new();
            const FTL_STRING: &str = include_str!(concat!(env!("OUT_DIR"), "/Counter.ftl"));
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
            bundle.format_pattern(pattern, args, &mut errors)
        }
    }
}
