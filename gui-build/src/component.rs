use crate::component_var::ComponentVars;
use crate::fluent;
use crate::fluent::FluentIdent;
use crate::widget::Widget;
use anyhow::{bail, Context};
use gui_core::parse::{ComponentDeclaration, StateDeclaration};
use itertools::Itertools;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use std::fs;
use std::path::Path;
use std::str::FromStr;

pub fn create_component(out_dir: &Path, component: &ComponentDeclaration) -> anyhow::Result<()> {
    let component_holder = format_ident!("{}Holder", *component.name);
    let component_name = format_ident!("{}", *component.name);

    let normal_variables = component
        .variables
        .iter()
        .filter_map(|v| v.get_normal())
        .collect_vec();

    let widget_tree = Widget::new(component)?;

    let component_vars = ComponentVars::new(&component.variables[..], &widget_tree)?;

    let mut widget_set = TokenStream::new();
    widget_tree.gen_widget_set(&mut widget_set);

    let mut fluents = vec![];
    widget_tree.push_fluents(&mut fluents);

    let bundle_func = (!fluents.is_empty()).then(|| fluent::gen_bundle_function(&component.name));

    create_bundle(out_dir, &component.name, &fluents[..])
        .context("Failed to create fluent bundle")?;

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

    let mut statics_update: TokenStream = TokenStream::new();
    widget_tree.gen_statics(None, &mut statics_update);

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

    let state_declaration = create_state(component.states.as_slice())?;

    let mut struct_handlers = TokenStream::new();
    widget_tree.gen_handler_structs(&mut struct_handlers)?;
    let comp_var_structs = component_vars.gen_comp_var_structs();
    let multi_comp = component_vars.gen_multi_comp();

    let rs_path = Path::new(&out_dir).join(format!("{}.rs", component.name.as_str()));

    let widget_type = widget_tree.gen_widget_type();
    let widget_init = widget_tree.gen_widget_init();

    let mut id_to_widgets = vec![];
    widget_tree.gen_widget_id_to_widget(None, &mut id_to_widgets);
    let event_match_arms = id_to_widgets.iter().map(|(widget_id, widget_get)| {
        let id = widget_id.id();
        quote!(#id => {#widget_get.event(event, handle_ref);})
    });

    let mut parent_ids = vec![];
    widget_tree.get_parent_ids(&mut parent_ids);
    let parent_match_arms = parent_ids.iter().map(|(parent, children)| {
        let ids = children.iter().map(|id| id.id());
        quote!(#( #ids )|* => Some(#parent),)
    });

    let named_match_arms = widget_tree.iter().filter_map(|w| {
        let name = w.widget_declaration.name.as_ref()?.as_str();
        let id = w.id;
        Some(quote!(#name => Some(#id),))
    });

    let check_state = state_declaration.as_ref().map(|_| {
        quote! {
            if force_update || <CompStruct as Update<state>>::is_updated(&self.comp_struct) {
                let new_state = <CompStruct as Update<state>>::value(&self.comp_struct);
                if self.state != new_state {
                    self.state = new_state;
                    force_update = true;
                }
            }
        }
    });
    let state_type = state_declaration.as_ref().map(|_| quote! {state: State,});
    let state_init = state_declaration
        .as_ref()
        .map(|_| quote! {state: Default::default(),});

    let gen_module = quote! {
        #[allow(clippy::suspicious_else_formatting)]
        #[allow(clippy::collapsible_if)]
        mod gen {
            use super::__private_CompStruct as CompStruct;
            use std::any::Any;
            use gui::gui_core::vello::SceneBuilder;
            use gui::gui_core::widget::{RuntimeID, Widget, WidgetID, RenderHandle, ResizeHandle, EventHandle, UpdateHandle, WidgetEvent, Handle};
            use gui::gui_core::{Component, ComponentHolder, ComponentTypeInfo, LayoutConstraints, MultiComponent, Size, ToComponent, ToHandler, Update, Variable};

            #state_declaration

            #widget_set

            #struct_vars

            #struct_handlers

            #comp_var_structs

            impl ComponentTypeInfo for crate::__gui_private::#component_name {
                type ToComponent = CompStruct;
            }

            #multi_comp

            #bundle_func

            #[allow(non_snake_case)]
            pub struct #component_holder {
                comp_struct: CompStruct,
                runtime_id: RuntimeID,
                widget: #widget_type,
                #state_type
                multi_comp: MultiComponentHolder,
                #( #fluent_arg_idents: FluentArgs<'static> ),*
            }

            #[automatically_derived]
            impl ToComponent for CompStruct {
                type Component = #component_holder;
                type HeldComponents = MultiComponentHolder;

                fn to_component_holder(mut self, runtime_id: RuntimeID) -> Self::Component {
                    #component_holder {
                        widget: #widget_init,
                        runtime_id,
                        multi_comp: MultiComponentHolder::new(&mut self),
                        comp_struct: self,
                        #state_init
                        #( #fluent_arg_idents: FluentArgs::new() ),*
                    }
                }

                fn get_parent(
                    &self,
                    widget_id: WidgetID,
                ) -> Option<WidgetID> {
                    match widget_id.id() {
                        #(#parent_match_arms)*
                        _ => None,
                    }
                }

                fn get_id(&self, name: &str) -> Option<WidgetID> {
                    match name {
                        #(#named_match_arms)*
                        _ => None,
                    }
                }
            }

            #[automatically_derived]
            impl Component for #component_holder {
                fn render(
                    &mut self,
                    scene: &mut SceneBuilder,
                    handle: &mut Handle,
                ) -> bool {
                    let mut render_handle = RenderHandle::new(handle, self.runtime_id, &mut self.comp_struct, &mut self.multi_comp);
                    self.widget.render(scene, &mut render_handle);
                    render_handle.unwrap()
                }

                #[allow(unused_mut)]
                fn update_vars(
                    &mut self,
                    mut force_update: bool,
                    handle: &mut Handle,
                ) -> bool {
                    let mut update_handle = UpdateHandle::new(handle, self.runtime_id);
                    let handle_ref = &mut update_handle;
                    #( let mut #fluent_properties = false; )*
                    #check_state
                    if force_update {
                        #statics_update
                    }
                    #if_update
                    #prop_update
                    #( <CompStruct as Update<#var_names>>::reset(&mut self.comp_struct); )*
                    update_handle.unwrap()
                }

                fn resize(
                    &mut self,
                    constraints: LayoutConstraints,
                    handle: &mut Handle,
                ) -> Size {
                    let mut resize_handle = ResizeHandle::new(handle, self.runtime_id, &mut self.comp_struct, &mut self.multi_comp);
                    self.widget.resize(constraints, &mut resize_handle)
                }

                fn propagate_event(
                    &mut self,
                    event: WidgetEvent,
                    handle: &mut Handle,
                ) -> bool {
                    let mut event_handle = EventHandle::new(handle, self.runtime_id, &mut self.comp_struct, &mut self.multi_comp);
                    self.widget.event(event, &mut event_handle);
                    let (mut resize, events) = event_handle.unwrap();
                    for (runtime_id, widget_id, e) in events {
                        if self.event(runtime_id, widget_id, e, handle) {
                            resize = true;
                        }
                    }
                    resize
                }

                fn get_parent(
                    &self,
                    runtime_id: RuntimeID,
                    widget_id: WidgetID,
                ) -> Option<(RuntimeID, WidgetID)> {
                    if runtime_id != self.runtime_id {
                        self.multi_comp.get_parent(runtime_id, widget_id)
                    } else {
                        self.comp_struct.get_parent(widget_id).map(|id| (self.runtime_id, id))
                    }
                }

                fn get_id(&self, name: &str) -> Option<(RuntimeID, WidgetID)> {
                    self.comp_struct.get_id(name)
                        .map_or_else(
                            || self.multi_comp.get_id(name),
                            |id| Some((self.runtime_id, id)),
                        )
                }

                fn get_comp_struct(&mut self) -> &mut dyn Any {
                    &mut self.comp_struct
                }

                fn event(
                    &mut self,
                    runtime_id: RuntimeID,
                    widget_id: WidgetID,
                    event: WidgetEvent,
                    handle: &mut Handle,
                ) -> bool {
                    if runtime_id != self.runtime_id {
                        return self.multi_comp.event(runtime_id, widget_id, event, handle);
                    }
                    let mut event_handle = EventHandle::new(handle, self.runtime_id, &mut self.comp_struct, &mut self.multi_comp);
                    let handle_ref = &mut event_handle;
                    match widget_id.id() {
                        #(#event_match_arms)*
                        _ => {},
                    }
                    let (mut resize, events) = event_handle.unwrap();
                    for (runtime_id, widget_id, e) in events {
                        if self.event(runtime_id, widget_id, e, handle) {
                            resize = true;
                        }
                    }
                    resize
                }
                fn id(&self) -> RuntimeID {
                    self.runtime_id
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
    let file = syn::parse2::<syn::File>(stream.clone()).inspect_err(|_| {
        let _ = fs::write(path, stream.to_string());
    })?;

    fs::write(path, prettyplease::unparse(&file))?;
    Ok(())
}

fn create_bundle(
    out_dir: &Path,
    component_name: &str,
    fluents: &[FluentIdent],
) -> anyhow::Result<()> {
    let ftl_path = out_dir.join(format!("{component_name}.ftl"));
    let mut bundle = String::new();
    for fluent in fluents {
        bundle = bundle + &format!("{} = {}\n", fluent.name, fluent.fluent.text);
    }
    fs::write(ftl_path, bundle)?;
    Ok(())
}

fn create_state(states: &[StateDeclaration]) -> anyhow::Result<Option<TokenStream>> {
    if states.is_empty() {
        return Ok(None);
    }

    if states.len() == 1 {
        let name = &states[0].name;
        bail!("Cannot have a singular state but found state {name}.");
    }

    let names = states.iter().map(|s| format_ident!("{}", s.name.as_str()));

    Ok(Some(quote! {
        #[allow(non_camel_case_types)]
        #[derive(Default, Copy, Clone, Eq, PartialEq)]
        pub(crate) enum State {
            #[default]
            #(#names),*
        }
        #[allow(non_camel_case_types)]
        pub(crate) struct state;

        impl Variable for state {
            type VarType = State;
        }
    }))
}
