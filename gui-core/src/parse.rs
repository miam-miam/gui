pub mod colour;
pub mod var;

pub mod fluent;

use crate::parse::var::Name;
use crate::widget::WidgetBuilder;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WidgetDeclaration {
    pub name: Option<Name>,
    #[serde(flatten)]
    pub widget: Box<dyn WidgetBuilder>,
    pub layout_properties: Option<LayoutDeclaration>,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct LayoutDeclaration {
    pub padding: u32,
}

#[derive(Deserialize, Debug, Eq, PartialEq, Hash, Clone)]
pub struct NormalVariableDeclaration {
    pub name: Name,
    #[serde(rename = "type")]
    pub var_type: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ComponentVariableDeclaration {
    pub name: Name,
    pub component: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ComponentsVariableDeclaration {
    pub name: Name,
    pub components: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum VariableDeclaration {
    Normal(NormalVariableDeclaration),
    Component(ComponentVariableDeclaration),
    Components(ComponentsVariableDeclaration),
}

impl VariableDeclaration {
    pub fn get_name(&self) -> &Name {
        match self {
            VariableDeclaration::Normal(v) => &v.name,
            VariableDeclaration::Component(c) => &c.name,
            VariableDeclaration::Components(c) => &c.name,
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct ComponentDeclaration {
    pub name: Name,
    #[serde(default)]
    pub variables: Vec<VariableDeclaration>,
    #[serde(default)]
    pub states: Vec<StateDeclaration>,
    pub child: WidgetDeclaration,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StateWidgetDeclaration {
    pub name: Name,
    #[serde(flatten)]
    pub widget: Box<dyn WidgetBuilder>,
    pub layout_properties: Option<LayoutDeclaration>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct StateDeclaration {
    pub name: Name,
    pub overrides: Vec<StateWidgetDeclaration>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GUIDeclaration {
    pub styles: Vec<Box<dyn WidgetBuilder>>,
    pub components: Vec<ComponentDeclaration>,
}
