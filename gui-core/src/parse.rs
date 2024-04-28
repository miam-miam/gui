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

#[derive(Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub struct LayoutDeclaration {
    pub padding: u32,
}

#[derive(Deserialize, Debug, Eq, PartialEq, Hash, Clone)]
pub struct NormalVariableDeclaration {
    pub name: Name,
    #[serde(rename = "type")]
    pub var_type: String,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq, Hash)]
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

    pub fn get_normal(&self) -> Option<&NormalVariableDeclaration> {
        if let VariableDeclaration::Normal(n) = self {
            Some(n)
        } else {
            None
        }
    }

    pub fn get_component(&self) -> Option<&ComponentVariableDeclaration> {
        if let VariableDeclaration::Component(c) = self {
            Some(c)
        } else {
            None
        }
    }

    pub fn get_components(&self) -> Option<&ComponentsVariableDeclaration> {
        if let VariableDeclaration::Components(c) = self {
            Some(c)
        } else {
            None
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
    #[serde(default)]
    pub styles: Vec<Box<dyn WidgetBuilder>>,
    pub components: Vec<ComponentDeclaration>,
}

#[cfg(test)]
mod test {
    use crate::parse::fluent::Fluent;
    use crate::parse::var::Name;
    use crate::parse::{GUIDeclaration, WidgetDeclaration};
    use crate::widget::{WidgetBuilder, WidgetID};
    use proc_macro2::{Ident, TokenStream};
    use serde::Deserialize;

    #[derive(Deserialize, Debug, Clone)]
    struct FakeWidget {
        number: Option<u32>,
    }

    #[allow(unused_variables)]
    #[typetag::deserialize(name = "FakeWidget")]
    impl WidgetBuilder for FakeWidget {
        fn widget_type(
            &self,
            handler: Option<&Ident>,
            component: &Ident,
            child: Option<&TokenStream>,
            stream: &mut TokenStream,
        ) {
            unimplemented!()
        }

        fn name(&self) -> &'static str {
            unimplemented!()
        }

        fn combine(&mut self, rhs: &dyn WidgetBuilder) {
            unimplemented!()
        }

        fn create_widget(
            &self,
            id: WidgetID,
            children: Option<&TokenStream>,
            stream: &mut TokenStream,
        ) {
            unimplemented!()
        }

        fn on_property_update(
            &self,
            property: &'static str,
            widget: &Ident,
            value: &Ident,
            handle: &Ident,
            stream: &mut TokenStream,
        ) {
            unimplemented!()
        }

        fn get_statics(&self) -> Vec<(&'static str, TokenStream)> {
            unimplemented!()
        }

        fn get_fluents(&self) -> Vec<(&'static str, Fluent)> {
            unimplemented!()
        }

        fn get_vars(&self) -> Vec<(&'static str, Name)> {
            unimplemented!()
        }

        fn get_widgets(&mut self) -> Option<Vec<&mut WidgetDeclaration>> {
            unimplemented!()
        }

        fn widgets(&self) -> Option<Vec<(TokenStream, &WidgetDeclaration)>> {
            unimplemented!()
        }
    }

    #[test]
    pub fn test_gui_declaration() {
        let yaml = r#"
styles:
  - widget: FakeWidget
    properties: 
      number: 20
components:
  - name: Component1
    variables:
      - name: component_variable
        component: Component3
      - name: variable
        type: u32
    child:
      widget: FakeWidget
      properties:
        number: 1

  - name: Component2 # Names for components must be unique
    states:
      - name: State1
        overrides:
          - name: FakeName
            widget: FakeWidget
            properties:
              number: 9

      - name: State2
        overrides:
          - name: FakeName
            widget: FakeWidget
            properties:
              number: 8

    child:
      name: FakeName
      widget: FakeWidget
      properties:
        number: 4
        "#;

        let decl = serde_yaml::from_str::<GUIDeclaration>(yaml).unwrap();

        // Assert that the number of components is 2
        assert_eq!(decl.components.len(), 2);

        // Assert that the first component's name is "Component1"
        assert_eq!(decl.components[0].name, "Component1".parse().unwrap());

        // Assert that the first component has 2 variables
        assert_eq!(decl.components[0].variables.len(), 2);

        // Assert that the second component's name is "Component2"
        assert_eq!(decl.components[1].name, "Component2".parse().unwrap());

        // Assert that the second component has 2 states
        assert_eq!(decl.components[1].states.len(), 2);

        // Assert that the first variable of the first component is a component variable
        let first_variable = decl.components[0].variables[0].get_component().unwrap();
        assert_eq!(first_variable.name, "component_variable".parse().unwrap());
        assert_eq!(first_variable.component, "Component3");

        // Assert that the second variable of the first component is a normal variable
        let second_variable = decl.components[0].variables[1].get_normal().unwrap();
        assert_eq!(second_variable.name, "variable".parse().unwrap());
        assert_eq!(second_variable.var_type, "u32");

        // Assert that the first state of the second component has the correct name and overrides
        let first_state = &decl.components[1].states[0];
        assert_eq!(first_state.name, "State1".parse().unwrap());
        assert_eq!(first_state.overrides[0].name, "FakeName".parse().unwrap());
        assert_eq!(first_state.overrides[0].layout_properties, None);
        let widget = first_state.overrides[0]
            .widget
            .as_ref()
            .as_any()
            .downcast_ref::<FakeWidget>()
            .unwrap();
        assert_eq!(widget.number, Some(9));

        // Assert that the second state of the second component has the correct name and overrides
        let second_state = &decl.components[1].states[1];
        assert_eq!(second_state.name, "State2".parse().unwrap());
        assert_eq!(second_state.overrides[0].name, "FakeName".parse().unwrap());
        assert_eq!(second_state.overrides[0].layout_properties, None);
        let widget = second_state.overrides[0]
            .widget
            .as_ref()
            .as_any()
            .downcast_ref::<FakeWidget>()
            .unwrap();
        assert_eq!(widget.number, Some(8));

        // Assert that the base widget tree of the first component has the correct name and overrides
        let widget = decl.components[0]
            .child
            .widget
            .as_ref()
            .as_any()
            .downcast_ref::<FakeWidget>()
            .unwrap();
        assert_eq!(decl.components[0].child.name, None);
        assert_eq!(decl.components[0].child.layout_properties, None);
        assert_eq!(widget.number, Some(1));

        // Assert that the base widget tree of the second component has the correct name and overrides
        let widget = decl.components[1]
            .child
            .widget
            .as_ref()
            .as_any()
            .downcast_ref::<FakeWidget>()
            .unwrap();
        assert_eq!(
            decl.components[1].child.name,
            Some("FakeName".parse().unwrap())
        );
        assert_eq!(decl.components[1].child.layout_properties, None);
        assert_eq!(widget.number, Some(4));

        // Assert that the styling is correct
        assert_eq!(decl.styles.len(), 1);
        let style = decl.styles[0]
            .as_ref()
            .as_any()
            .downcast_ref::<FakeWidget>()
            .unwrap();
        assert_eq!(style.number, Some(20));
    }
}
