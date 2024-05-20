use std::borrow::Cow;

use gui_core::widget::{RuntimeID, UpdateHandle, WidgetID};

pub fn init_path<W>(_: impl Fn(WidgetID) -> W) {}

pub fn property_path<W, T>(_: impl Fn(&mut W, T, &mut UpdateHandle)) {}

pub fn fluent_path<W>(_: impl Fn(&mut W, Cow<str>, &mut UpdateHandle)) {}

pub fn component_path<W>(_: impl Fn(&mut W, RuntimeID, &mut UpdateHandle)) {}

pub fn child_path<W, C>(_: impl Fn(&mut W) -> &mut Option<C>) {}

pub fn children_path<W, C>(_: impl Fn(&mut W) -> &mut Vec<C>) {}