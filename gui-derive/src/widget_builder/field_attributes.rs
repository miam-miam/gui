use proc_macro2::Ident;
use syn::spanned::Spanned;
use syn::{Error, Expr, Field, Path, PredicateType};

use crate::widget_builder::attributes::{
    get_attributes, parse_from_lit, require_func_path, require_type_predicate,
};

#[derive(Clone, Debug)]
pub struct FieldAttributes {
    pub field: Field,
    pub static_default: Option<StaticDefault>,
    pub static_prop: Option<Path>,
    pub static_bound: Option<PredicateType>,
    pub var_prop: Option<Path>,
    pub var_bound: Option<PredicateType>,
    pub fluent: Option<Path>,
    pub component: Option<Path>,
    pub child: Option<Path>,
    pub children: Option<Path>,
}

/// Only check ident in field for equality
impl PartialEq for FieldAttributes {
    fn eq(&self, other: &FieldAttributes) -> bool {
        self.field.ident == other.field.ident
            && self.static_default == other.static_default
            && self.static_prop == other.static_prop
            && self.var_prop == other.var_prop
            && self.fluent == other.fluent
            && self.component == other.component
            && self.child == other.child
            && self.children == other.children
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Extension {
    Unnecessary(Property),
    /// If the field also contains a var
    Static(bool),
    /// If the field also contains a static
    Var(bool),
    Fluent,
    Component,
}

impl Extension {
    pub fn to_str(self) -> &'static str {
        match self {
            Extension::Unnecessary(_) => "",
            Extension::Static(_) => "_static",
            Extension::Var(_) => "_var",
            Extension::Fluent => "_fluent",
            Extension::Component => "_component",
        }
    }

    pub fn form_prop_name(self, ident: &Ident) -> String {
        format!("{}{}", ident, self.to_str())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Property {
    Static,
    Var,
    /// Var and Static
    Both,
    Fluent,
    Component,
}

impl From<Extension> for Property {
    fn from(value: Extension) -> Self {
        match value {
            Extension::Unnecessary(p) => p,
            Extension::Static(_) => Property::Static,
            Extension::Var(_) => Property::Var,
            Extension::Fluent => Property::Fluent,
            Extension::Component => Property::Component,
        }
    }
}

impl From<&Extension> for Property {
    fn from(value: &Extension) -> Self {
        Self::from(*value)
    }
}

impl Property {
    pub fn is_static(self) -> bool {
        matches!(self, Property::Static | Property::Both)
    }

    pub fn is_var(self) -> bool {
        matches!(self, Property::Var | Property::Both)
    }
}

impl FieldAttributes {
    pub fn property_names(&self) -> Vec<(Extension, Option<&StaticDefault>, &Path)> {
        match (
            &self.static_prop,
            &self.var_prop,
            &self.fluent,
            &self.component,
        ) {
            (Some(static_prop), None, None, None) => {
                return vec![(
                    Extension::Unnecessary(Property::Static),
                    self.static_default.as_ref(),
                    static_prop,
                )];
            }
            (None, Some(var_prop), None, None) => {
                return vec![(Extension::Unnecessary(Property::Var), None, var_prop)];
            }
            (Some(static_prop), Some(var_prop), _, _) if static_prop == var_prop => {
                let mut result = vec![(
                    Extension::Unnecessary(Property::Both),
                    self.static_default.as_ref(),
                    static_prop,
                )];
                if let Some(fluent) = &self.fluent {
                    result.push((Extension::Fluent, None, fluent));
                }
                if let Some(component) = &self.component {
                    result.push((Extension::Component, None, component));
                }
                return result;
            }
            (None, None, Some(fluent), None) => {
                return vec![(Extension::Unnecessary(Property::Fluent), None, fluent)];
            }
            (None, None, None, Some(component)) => {
                return vec![(Extension::Unnecessary(Property::Component), None, component)]
            }
            _ => {}
        }

        let mut result = vec![];
        let contains_var_and_static = self.static_prop.is_some() && self.var_prop.is_some();

        if let Some(static_prop) = &self.static_prop {
            result.push((
                Extension::Static(contains_var_and_static),
                self.static_default.as_ref(),
                static_prop,
            ));
        }
        if let Some(var_prop) = &self.var_prop {
            result.push((Extension::Var(contains_var_and_static), None, var_prop));
        }
        if let Some(fluent) = &self.fluent {
            result.push((Extension::Fluent, None, fluent));
        }
        if let Some(component) = &self.component {
            result.push((Extension::Component, None, component));
        }

        result
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum StaticDefault {
    Expression(Expr),
    Function(Path),
}

impl FieldAttributes {
    pub fn new(field: Field) -> syn::Result<Self> {
        let mut static_default = None;
        let mut static_prop = None;
        let mut static_bound = None;
        let mut var_prop = None;
        let mut var_bound = None;
        let mut fluent = None;
        let mut component = None;
        let mut child = None;
        let mut children = None;

        let attributes = get_attributes(&field.attrs)?;

        for (name, expr) in attributes {
            match name.to_string().as_str() {
                "property" if static_prop.is_none() && var_prop.is_none() => {
                    static_prop = Some(require_func_path(parse_from_lit(expr)?)?);
                    var_prop.clone_from(&static_prop);
                }
                "property_bound" if static_bound.is_none() && var_bound.is_none() => {
                    static_bound = Some(require_type_predicate(parse_from_lit(expr)?)?);
                    var_bound.clone_from(&static_bound);
                }
                "static_only" if static_prop.is_none() => {
                    static_prop = Some(require_func_path(parse_from_lit(expr)?)?)
                }
                "static_bound" if static_bound.is_none() => {
                    static_bound = Some(require_type_predicate(parse_from_lit(expr)?)?)
                }
                "var_only" if var_prop.is_none() => {
                    var_prop = Some(require_func_path(parse_from_lit(expr)?)?)
                }
                "var_bound" if static_bound.is_none() => {
                    var_bound = Some(require_type_predicate(parse_from_lit(expr)?)?)
                }
                "fluent" if fluent.is_none() => {
                    fluent = Some(require_func_path(parse_from_lit(expr)?)?)
                }
                "component" if component.is_none() => {
                    component = Some(require_func_path(parse_from_lit(expr)?)?)
                }
                "child" if child.is_none() => {
                    child = Some(require_func_path(parse_from_lit(expr)?)?)
                }
                "children" if children.is_none() => {
                    children = Some(require_func_path(parse_from_lit(expr)?)?)
                }
                "default" if static_default.is_none() => {
                    static_default = Some(StaticDefault::Expression(expr))
                }
                "default_with" if static_default.is_none() => {
                    static_default = Some(StaticDefault::Function(require_func_path(
                        parse_from_lit(expr)?,
                    )?))
                }
                _ => return Err(Error::new(name.span(), "Unexpected attribute")),
            }
        }

        if !(static_prop.is_some()
            || var_prop.is_some()
            || fluent.is_some()
            || component.is_some()
            || child.is_some()
            || children.is_some())
        {
            Err(Error::new(
                field.ident.span(),
                "Expected at least one widget attribute on this field",
            ))
        } else if static_default.is_some() && static_prop.is_none() {
            Err(Error::new(
                field.ident.span(),
                "Defaults only apply to static properties",
            ))
        } else if static_bound.is_some() && static_prop.is_none() {
            Err(Error::new(
                field.ident.span(),
                "Bounds only apply to static properties",
            ))
        } else if var_bound.is_some() && var_prop.is_none() {
            Err(Error::new(
                field.ident.span(),
                "Bounds only apply to var properties",
            ))
        } else {
            Ok(FieldAttributes {
                field,
                static_default,
                static_prop,
                static_bound,
                var_prop,
                var_bound,
                fluent,
                component,
                child,
                children,
            })
        }
    }
}
