use crate::fluent;
use crate::fluent::FluentIdent;
use crate::widget::Widget;
use anyhow::anyhow;
use gui_core::parse::{ComponentDeclaration, NormalVariableDeclaration, VariableDeclaration};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use std::fs;
use std::path::Path;
use std::str::FromStr;

pub fn create_component(out_dir: &Path, component: &ComponentDeclaration) -> anyhow::Result<()> {
    let normal_variables: Vec<&NormalVariableDeclaration> = component
        .variables
        .iter()
        .filter_map(|v| match v {
            VariableDeclaration::Normal(n) => Some(n),
            _ => None,
        })
        .collect();

    let widget_tree = Widget::new(component)?;

    let bundle_func = widget_tree
        .contains_fluents()
        .then(fluent::gen_bundle_function);

    let mut fluents = vec![];
    widget_tree.push_fluents(&mut fluents);

    create_bundle(out_dir, &component.name, &fluents[..])?;

    let if_update: TokenStream = normal_variables
        .iter()
        .map(|n| widget_tree.gen_var_update(n))
        .collect();

    let fluent_arg_idents: Vec<&Ident> = fluents
        .iter()
        .filter(|f| !f.fluent.vars.is_empty())
        .map(|fluent| &fluent.ident)
        .collect();

    let fluent_properties: Vec<&Ident> = fluents
        .iter()
        .filter(|f| !f.fluent.vars.is_empty())
        .map(|fluent| &fluent.property_ident)
        .collect();

    let mut prop_update: TokenStream = TokenStream::new();
    widget_tree.gen_fluent_update(None, &mut prop_update);

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

    let var_names = normal_variables
        .iter()
        .map(|n| Ident::new(&n.name, Span::call_site()));

    let struct_handlers = gen_handler_structs(component)?;

    let component_holder = Ident::new(&format!("{}Holder", component.name), Span::call_site());

    let rs_path = Path::new(&out_dir).join(format!("{}.rs", component.name));

    let widget_type = widget_tree.gen_widget_type();
    let widget_init = widget_tree.gen_widget_init();

    let gen_module = quote! {
        #[allow(clippy::suspicious_else_formatting)]
        mod gen {
            use super::__private_CompStruct as CompStruct;
            use gui::gui_core::glazier::{PointerEvent, WindowHandle};
            use gui::gui_core::parley::font::FontContext;
            use gui::gui_core::vello::SceneBuilder;
            use gui::gui_core::widget::Widget;
            use gui::gui_core::{Component, LayoutConstraints, Size, ToComponent, ToHandler, Update, Variable};

            #bundle_func

            #struct_vars

            #struct_handlers

            #[allow(non_snake_case)]
            pub struct #component_holder {
                comp_struct: CompStruct,
                widget: #widget_type,
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
                fn render(&mut self, mut scene: SceneBuilder, fcx: &mut FontContext) {
                    self.widget.render(&mut scene, fcx);
                }

                fn update_vars(&mut self, force_update: bool) {
                    #( let mut #fluent_properties = false; )*
                    #if_update
                    #prop_update
                    #( <CompStruct as Update<#var_names>>::reset(&mut self.comp_struct); )*
                }

                fn resize(&mut self, constraints: LayoutConstraints, fcx: &mut FontContext) -> Size{
                    self.widget.resize(constraints, fcx)
                }

                fn pointer_down(&mut self, event: &PointerEvent, window: &WindowHandle) {
                    self.widget.pointer_down(event, window, &mut self.comp_struct);
                }

                fn pointer_up(&mut self, event: &PointerEvent, window: &WindowHandle) {
                    self.widget.pointer_up(event, window, &mut self.comp_struct);
                }

                fn pointer_move(&mut self, event: &PointerEvent, window: &WindowHandle) {
                    self.widget.pointer_move(event, window, &mut self.comp_struct);
                }
            }
        }
    };

    write_file(&rs_path, gen_module)
}

#[cfg(not(feature = "pretty"))]
fn write_file(path: &Path, stream: TokenStream) -> anyhow::Result<()> {
    fs::write(path, format!("{}", stream))?;
    Ok(())
}
#[cfg(feature = "pretty")]
fn write_file(path: &Path, stream: TokenStream) -> anyhow::Result<()> {
    let file = syn::parse2::<syn::File>(stream)?;

    fs::write(path, prettyplease::unparse(&file))?;
    Ok(())
}

fn gen_handler_structs(component: &ComponentDeclaration) -> anyhow::Result<TokenStream> {
    Ok(if component.child.widget.has_handler() {
        let name = Ident::new(
            component
                .child
                .name
                .as_ref()
                .ok_or_else(|| anyhow!("Widgets with handlers must be named."))?,
            Span::call_site(),
        );
        quote! {
            pub(crate) struct #name;

            impl ToHandler for #name {
                type BaseHandler = CompStruct;
            }
        }
    } else {
        quote!()
    })
}

fn create_bundle(
    out_dir: &Path,
    component_name: &str,
    fluents: &[FluentIdent],
) -> anyhow::Result<()> {
    let ftl_path = out_dir.join(format!("{component_name}.ftl"));
    let mut bundle = String::new();
    for fluent in fluents {
        bundle = bundle + &format!("{} = {}", fluent.name, fluent.fluent.text);
    }
    fs::write(ftl_path, bundle)?;
    Ok(())
}
