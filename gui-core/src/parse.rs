pub mod colour;
pub mod var;

pub mod fluent;

use crate::widget::WidgetBuilder;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct WidgetDeclaration {
    pub name: Option<String>,
    #[serde(flatten)]
    pub widget: Box<dyn WidgetBuilder>,
    pub layout_properties: Option<LayoutDeclaration>,
}

#[derive(Deserialize, Debug)]
pub struct LayoutDeclaration {
    pub padding: u32,
}

#[derive(Deserialize, Debug, Eq, PartialEq, Hash)]
pub struct NormalVariableDeclaration {
    pub name: String,
    #[serde(rename = "type")]
    pub var_type: String,
}

#[derive(Deserialize, Debug)]
pub struct ComponentVariableDeclaration {
    pub name: String,
    pub component: String,
}

#[derive(Deserialize, Debug)]
pub struct ComponentsVariableDeclaration {
    pub name: String,
    pub components: String,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum VariableDeclaration {
    Normal(NormalVariableDeclaration),
    Component(ComponentVariableDeclaration),
    Components(ComponentsVariableDeclaration),
}

#[derive(Deserialize, Debug)]
pub struct ComponentDeclaration {
    pub name: String,
    #[serde(default)]
    pub variables: Vec<VariableDeclaration>,
    pub child: WidgetDeclaration,
}

#[derive(Deserialize, Debug)]
pub struct GUIDeclaration {
    pub styles: Vec<Box<dyn WidgetBuilder>>,
    pub components: Vec<ComponentDeclaration>,
}
