use crate::widget::Widget;
use anyhow::bail;
use gui_core::parse::var::Name;
use gui_core::parse::{ComponentVariableDeclaration, VariableDeclaration};
use gui_core::widget::WidgetID;
use itertools::Itertools;
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct ComponentVar {
    type_stream: TokenStream,
    comp_name: String,
    holder_ident: Ident,
    name: Name,
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
            comp_name: comp.component.clone(),
            holder_ident,
            name: comp.name.clone(),
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
        for (_, name, id) in widget_tree.iter().flat_map(|w| w.components.0.iter()) {
            match component_variables.get(name) {
                None => {
                    bail!("Could not find variable {name} in component variables")
                }
                Some(&comp_decl) => {
                    if component_map.insert(name, (id, comp_decl)).is_some() {
                        bail!("Cannot have {name} component variable used multiple times.")
                    }
                }
            }
        }
        Ok(ComponentVars(
            component_map
                .into_iter()
                .map(|(_, (id, comp_decl))| ComponentVar::new(comp_decl, *id))
                .collect_vec(),
        ))
    }

    pub fn gen_multi_comp(&self) -> TokenStream {
        let component_idents = self.0.iter().map(|c| &c.holder_ident).collect_vec();
        let component_names = self.0.iter().map(|c| &c.name_ident).collect_vec();
        let component_types = self.0.iter().map(|c| &c.type_stream).collect_vec();

        let render = self.gen_match_multi(
            quote!(render(
                scene,
                handle,
                global_positions,
                active_widget,
                hovered_widgets
            )),
            quote!(false),
        );
        let update_vars = self.gen_match_multi(
            quote!(update_vars(force_update, handle, global_positions)),
            quote!(false),
        );
        let resize = self.gen_match_multi(
            quote!(resize(constraints, handle, local_positions)),
            quote!(Size::ZERO),
        );
        let propagate_event = self.gen_match_multi(
            quote!(propagate_event(
                event,
                handle,
                global_positions,
                active_widget,
                hovered_widgets
            )),
            quote!(false),
        );
        let event = self.gen_match_multi(
            quote!(event(
                id,
                event,
                handle,
                global_positions,
                active_widget,
                hovered_widgets
            )),
            quote!(false),
        );

        quote! {
            pub struct MultiComponent {
                #( #component_idents: <#component_types::ToComponent as ToComponent>::Component),*
            }

            impl MultiComponent {
                pub fn new(comp: &mut CompStruct) -> Self {
                    #(
                        let comp_holder = <CompStruct as ComponentHolder<#component_names>>::comp_holder(comp);
                        let #component_idents = comp_holder.take().expect("Component is initialised.").to_component_holder();
                    )*
                    Self {
                        #(#component_idents),*
                    }
                }
            }

            impl MultiComponentHolder for MultiComponent {
                fn render(
                    &mut self,
                    comp_id: WidgetID,
                    scene: &mut SceneBuilder,
                    handle: &mut Handle,
                    global_positions: &mut [Rect],
                    active_widget: &mut Option<WidgetID>,
                    hovered_widgets: &[WidgetID],
                ) -> bool {
                    #render
                }
                fn update_vars(
                    &mut self,
                    comp_id: WidgetID,
                    force_update: bool,
                    handle: &mut Handle,
                    global_positions: &[Rect],
                ) -> bool {
                    #update_vars
                }
                fn resize(
                    &mut self,
                    comp_id: WidgetID,
                    constraints: LayoutConstraints,
                    handle: &mut Handle,
                    local_positions: &mut [Rect],
                ) -> Size {
                    #resize
                }
                fn propagate_event(
                    &mut self,
                    comp_id: WidgetID,
                    event: WidgetEvent,
                    handle: &mut Handle,
                    global_positions: &[Rect],
                    active_widget: &mut Option<WidgetID>,
                    hovered_widgets: &mut Vec<WidgetID>,
                ) -> bool {
                    #propagate_event
                }
                fn event(
                    &mut self,
                    comp_id: WidgetID,
                    id: WidgetID,
                    event: WidgetEvent,
                    handle: &mut Handle,
                    global_positions: &[Rect],
                    active_widget: &mut Option<WidgetID>,
                    hovered_widgets: &mut Vec<WidgetID>,
                ) -> bool {
                    #event
                }
            }
        }
    }

    fn gen_match_multi(&self, stream: TokenStream, default: TokenStream) -> TokenStream {
        let guards = self.0.iter().map(|c| {
            let holder_ident = &c.holder_ident;
            let widget_id = c.id.widget_id();
            Some(quote! {
                #widget_id => self.#holder_ident.#stream,
            })
        });
        quote! {
            match id.widget_id() {
                #(#guards)*
                _ => #default
            }
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
