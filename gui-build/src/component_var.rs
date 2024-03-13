use crate::widget::Widget;
use anyhow::bail;
use gui_core::parse::{ComponentVariableDeclaration, VariableDeclaration};
use gui_core::widget::WidgetID;
use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::collections::HashMap;
use std::iter;

#[derive(Debug, Clone)]
struct ComponentVar {
    type_stream: TokenStream,
    holder_ident: Ident,
    name_ident: Ident,
    id: WidgetID,
}

impl ComponentVar {
    pub fn new(comp: &ComponentVariableDeclaration, id: WidgetID) -> Self {
        let comp_name_ident = format_ident!("{}", *comp.component);
        let type_stream = quote!(<crate::__gui_private::#comp_name_ident as ComponentTypeInfo>);
        let holder_ident = format_ident!("{}_holder", *comp.name);
        let name_ident = format_ident!("{}", *comp.name);
        Self {
            type_stream,
            holder_ident,
            name_ident,
            id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ComponentVars(Vec<ComponentVar>);

impl ComponentVars {
    pub fn new(variables: &[VariableDeclaration], widget_tree: &Widget) -> anyhow::Result<Self> {
        let component_variables: HashMap<_, _> = variables
            .iter()
            .filter_map(|c| c.get_component())
            .map(|c| (&c.name, c))
            .collect();

        let mut component_map = HashMap::new();
        for ((_, name), id) in widget_tree
            .iter()
            .flat_map(|w| w.components.0.iter().zip(iter::repeat(w.id)))
        {
            match component_variables.get(name) {
                None => {
                    bail!("Could not find variable {name} in component variables")
                }
                Some(&comp_decl) => {
                    if component_map.insert(name, (comp_decl, id)).is_some() {
                        bail!("Cannot have {name} component variable used multiple times.")
                    }
                }
            }
        }
        Ok(ComponentVars(
            component_map
                .into_values()
                .map(|(comp_decl, id)| ComponentVar::new(comp_decl, id))
                .collect_vec(),
        ))
    }

    pub fn gen_multi_comp(&self) -> TokenStream {
        let component_idents = self.0.iter().map(|c| &c.holder_ident).collect_vec();
        let component_names = self.0.iter().map(|c| &c.name_ident).collect_vec();
        let component_types = self.0.iter().map(|c| &c.type_stream).collect_vec();

        let render = self.gen_match_multi(quote!(render(scene, handle)), quote!(false));
        let update_vars =
            self.gen_match_multi(quote!(update_vars(force_update, handle)), quote!(false));
        let force_update_vars =
            self.gen_for_each_comp(quote!(update_vars(true, handle)), quote!(false));
        let resize = self.gen_match_multi(quote!(resize(constraints, handle)), quote!(Size::ZERO));
        let propagate_event =
            self.gen_match_multi(quote!(propagate_event(event, handle)), quote!(false));
        let event = self.gen_for_each_comp(
            quote!(event(runtime_id, widget_id, event.clone(), handle)),
            quote!(false),
        );
        let get_parent = self.gen_try_all_options(quote!(get_parent(runtime_id, widget_id)));
        let get_id = self.gen_try_all_options(quote!(get_id(name)));
        let get_parent_runtime = self.gen_get_parent_runtime();

        quote! {
            pub struct MultiComponentHolder {
                #[allow(dead_code)]
                parent_id: RuntimeID,
                #( #component_idents: <#component_types::ToComponent as ToComponent>::Component),*
            }

            #[automatically_derived]
            impl MultiComponentHolder {
                pub fn new(comp: &mut CompStruct, parent_id: RuntimeID) -> Self {
                    #(
                        let comp_holder = <CompStruct as ComponentHolder<#component_names>>::comp_holder(comp);
                        let #component_idents = comp_holder.take().expect("Component is initialised.").to_component_holder(RuntimeID::next());
                    )*
                    Self {
                        parent_id,
                        #(#component_idents),*
                    }
                }
            }

            #[automatically_derived]
            impl MultiComponent for MultiComponentHolder {
                fn render(
                    &mut self,
                    runtime_id: RuntimeID,
                    scene: &mut SceneBuilder,
                    handle: &mut Handle,
                ) -> bool {
                    #render
                }
                fn update_vars(
                    &mut self,
                    runtime_id: RuntimeID,
                    force_update: bool,
                    handle: &mut Handle,
                ) -> bool {
                    #update_vars
                }
                fn force_update_vars(&mut self, handle: &mut Handle) -> bool {
                    #force_update_vars
                }
                fn resize(
                    &mut self,
                    runtime_id: RuntimeID,
                    constraints: LayoutConstraints,
                    handle: &mut Handle,
                ) -> Size {
                    #resize
                }
                fn propagate_event(
                    &mut self,
                    runtime_id: RuntimeID,
                    event: WidgetEvent,
                    handle: &mut Handle,
                ) -> bool {
                    #propagate_event
                }
                fn event(
                    &mut self,
                    runtime_id: RuntimeID,
                    widget_id: WidgetID,
                    event: WidgetEvent,
                    handle: &mut Handle,
                ) -> bool {
                    #event
                }
                fn get_parent(
                    &self,
                    runtime_id: RuntimeID,
                    widget_id: WidgetID,
                ) -> Option<(RuntimeID, WidgetID)> {
                    #get_parent_runtime
                    #get_parent
                }
                fn get_id(&self, name: &str) -> Option<(RuntimeID, WidgetID)> {
                    #get_id
                }
            }
        }
    }

    fn gen_match_multi(&self, stream: TokenStream, default: TokenStream) -> TokenStream {
        let guards = self.0.iter().map(|c| {
            let holder_ident = &c.holder_ident;
            Some(quote! {
                id if id == self.#holder_ident.id() => self.#holder_ident.#stream,
            })
        });
        quote! {
            match runtime_id {
                #(#guards)*
                _ => #default
            }
        }
    }

    fn gen_try_all_options(&self, stream: TokenStream) -> TokenStream {
        self.0.first().map_or_else(
            || quote!(None),
            |_| {
                self.0
                    .iter()
                    .fold(None, |acc, c| {
                        let holder_ident = &c.holder_ident;
                        let acc = acc.map(|acc| quote!(.or_else(|| #acc))).unwrap_or_default();
                        Some(quote!(self.#holder_ident . #stream #acc))
                    })
                    .expect("has first")
            },
        )
    }

    fn gen_get_parent_runtime(&self) -> TokenStream {
        let result: TokenStream = self
            .0
            .iter()
            .map(|c| {
                let id = c.id;
                let holder_ident = &c.holder_ident;
                quote! {
                    if self.#holder_ident.id() == runtime_id {
                        return Some((self.parent_id, #id))
                    }
                }
            })
            .collect();
        if result.is_empty() {
            result
        } else {
            quote! {
                if widget_id.id() == 0 {
                    #result
                }
            }
        }
    }

    fn gen_for_each_comp(&self, stream: TokenStream, default: TokenStream) -> TokenStream {
        if self.0.is_empty() {
            return default;
        }
        let component_idents = self.0.iter().map(|c| &c.holder_ident).collect_vec();
        quote! {
            #(let #component_idents = self.#component_idents.#stream;)*
            #(#component_idents)||*
        }
    }

    pub fn gen_comp_var_structs(&self) -> TokenStream {
        self.0
            .iter()
            .map(|c| {
                let name = &c.name_ident;
                let component_type = &c.type_stream;
                quote! {
                    #[allow(non_camel_case_types)]
                    pub(crate) struct #name;

                    impl Variable for #name {
                        type VarType = #component_type::ToComponent;
                    }
                }
            })
            .collect()
    }
}
