use syn::{Error, Expr, Field, Path};
use syn::spanned::Spanned;

use crate::widget_builder::attributes::{get_attributes, parse_from_lit, require_func_path};
use crate::widget_builder::field_attributes::StaticDefault::{Expression, Function};

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct FieldAttributes {
    static_default: Option<StaticDefault>,
    static_prop: Option<Path>,
    var_prop: Option<Path>,
    fluent: Option<Path>,
    component: Option<Path>,
    child: Option<Path>,
    children: Option<Path>,
}

#[derive(Clone, Eq, PartialEq, Debug)]
enum StaticDefault {
    Expression(Expr),
    Function(Path),
}

impl FieldAttributes {
    pub fn new(field: Field) -> syn::Result<Self> {
        let mut static_default = None;
        let mut static_prop = None;
        let mut var_prop = None;
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
                "static_only" if static_prop.is_none() => {
                    static_prop = Some(require_func_path(parse_from_lit(expr)?)?)
                }
                "var_only" if var_prop.is_none() => {
                    var_prop = Some(require_func_path(parse_from_lit(expr)?)?)
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
                "default" if static_default.is_none() => static_default = Some(Expression(expr)),
                "default_with" if static_default.is_none() => {
                    static_default = Some(Function(require_func_path(parse_from_lit(expr)?)?))
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
            Err(Error::new(field.span(), "Expected at least one attribute"))
        } else if static_default.is_some() && static_prop.is_none() {
            Err(Error::new(
                field.span(),
                "Defaults only apply to static properties",
            ))
        } else {
            Ok(FieldAttributes {
                static_default,
                static_prop,
                var_prop,
                fluent,
                component,
                child,
                children,
            })
        }
    }
}
