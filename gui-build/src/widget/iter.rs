use crate::widget::Widget;

pub struct WidgetIter<'a, 'b> {
    widget: Option<&'b Widget<'a>>,
    to_visit: Vec<&'b Widget<'a>>,
}

impl<'a, 'b> WidgetIter<'a, 'b> {
    pub fn new(widget: &'b Widget<'a>) -> Self {
        Self {
            widget: Some(widget),
            to_visit: vec![],
        }
    }
}

impl<'a, 'b> Iterator for WidgetIter<'a, 'b> {
    type Item = &'b Widget<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(Some(ws)) = &self.widget.map(|w| &w.child_widgets) {
            self.to_visit.extend(ws.widgets.iter().map(|(_, w)| w));
        }
        let widget = self.widget;
        self.widget = self.to_visit.pop();
        widget
    }
}

#[cfg(test)]
mod test {
    use crate::widget::Widget;
    use gui_core::parse::ComponentDeclaration;
    use itertools::Itertools;

    #[test]
    fn test_iter() -> anyhow::Result<()> {
        let declaration: ComponentDeclaration = serde_yaml::from_str(
            r#"
name: Test
child:
  widget: VStack
  properties:
    children:
      - name: One
        widget: Text
        properties:
          text: a

      - name: Two
        widget: HStack
        properties:
          children:
            - name: TwoA
              widget: Text
              properties:
                text: a
            - name: TwoB
              widget: Text
              properties:
                text: a

      - name: Three
        widget: Button
        properties:
          child:
            name: Four
            widget: Text
            properties:
              text: a

      - name: Five
        widget: Button
        properties:
          child:
            name: Six
            widget: Button
            properties:
              child:
                name: Seven
                widget: Text
                properties:
                  text: a
        "#,
        )?;
        let tree = Widget::new(&declaration)?;
        let mut v = tree
            .iter()
            .filter_map(|w| w.widget_declaration.name.as_ref().map(|s| s.as_str()))
            .collect_vec();
        let mut slice = [
            "One", "Two", "TwoA", "TwoB", "Three", "Four", "Five", "Six", "Seven",
        ];
        slice.sort();
        v.sort();
        assert_eq!(v, slice);
        Ok(())
    }
}
