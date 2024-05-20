use itertools::Itertools;
use proc_macro2::{Ident, Span};
use syn::{Attribute, Error, Expr, Lit, MetaNameValue, Path, Token};
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;

use crate::widget_builder::interpolated_path::{InterpolatedPath, InterpolatedType};

pub fn get_attributes(attributes: &[Attribute]) -> syn::Result<Vec<(Ident, Expr)>> {
    attributes
        .iter()
        .filter(|a| a.path().is_ident("widget"))
        .map(|a| {
            let values =
                a.parse_args_with(Punctuated::<MetaNameValue, Token![,]>::parse_terminated)?;
            values
                .iter()
                .map(|meta| {
                    let ident = meta.path.require_ident()?;
                    Ok((ident.clone(), meta.value.clone()))
                })
                .collect::<syn::Result<Vec<_>>>()
        })
        .flatten_ok()
        .collect()
}

pub fn parse_from_lit<T: Parse>(expr: Expr) -> syn::Result<T> {
    if let Expr::Lit(lit) = &expr {
        if let Lit::Str(str) = &lit.lit {
            return str.parse();
        }
    }
    Err(Error::new(expr.span(), "Expected literal string"))
}

pub fn require_lit(expr: Expr) -> syn::Result<String> {
    if let Expr::Lit(lit) = &expr {
        if let Lit::Str(str) = &lit.lit {
            return Ok(str.value())
        }
    }
    Err(Error::new(expr.span(), "Expected literal string"))
}

pub fn require_func_path(path: Path) -> syn::Result<Path> {
    if path.leading_colon.is_some() {
        Err(Error::new(path.leading_colon.span(), "Unexpected leading colon"))
    } else if path.segments.len() > 1 {
        Err(Error::new(path.segments.span(), "Expected a function name"))
    } else {
        Ok(path)
    }
}

pub fn require_attribute<T>(
    attribute: Option<T>,
    span: impl FnOnce() -> Span,
    name: &str,
) -> syn::Result<T> {
    match attribute {
        None => Err(Error::new(span(), format!("Expected to find a {name} attribute of the form #[widget({name} = ...)]"))),
        Some(a) => Ok(a),
    }
}

/// Returns whether the path contains a handler
pub fn check_interpolated_path(path: &InterpolatedPath) -> syn::Result<bool> {
    if path.generics.is_none() {
        return Ok(false)
    }

    let mut has_handler = false;
    for arg in &path.generics.as_ref().unwrap().args {
        if let InterpolatedType::Interpolated { name, .. } = arg {
            match name.to_string().as_str() {
                "handler" => {
                    has_handler = true;
                },
                "component" | "child" => {},
                name => {
                    return Err(Error::new(Span::call_site(), format!("Unexpected attribute {name}")));
                }
            }
        }
    }

    Ok(has_handler)
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct StructAttributes {
    pub widget_name: String,
    pub has_handler: bool,
    pub type_path: InterpolatedPath,
    pub init_path: Path,
}

impl StructAttributes {
    pub fn new(attributes: &[Attribute]) -> syn::Result<Self> {
        let attributes = get_attributes(attributes)?;

        let mut widget_name = None;
        let mut type_path = None;
        let mut init_path = None;

        for (name, expr) in attributes {
            match name.to_string().as_str() {
                "name" if widget_name.is_none() => widget_name = Some(require_lit(expr)?),
                "type_path" if type_path.is_none() => type_path = Some(parse_from_lit(expr)?),
                "init_path" if init_path.is_none() => init_path = Some(require_func_path(parse_from_lit(expr)?)?),
                _ => return Err(Error::new(name.span(), "Unexpected attribute")),
            }
        }

        let widget_name = require_attribute(widget_name, Span::call_site, "name")?;
        let type_path = require_attribute(type_path, Span::call_site, "type_path")?;
        let init_path = require_attribute(init_path, Span::call_site, "init_path")?;

        let has_handler = check_interpolated_path(&type_path)?;

        Ok(StructAttributes {
            has_handler,
            widget_name,
            type_path,
            init_path,
        })
    }
}