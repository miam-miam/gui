use crate::{OnMessage, ToComponent};

#[derive(Clone, Debug)]
pub struct CompHolder<T: ToComponent + OnMessage> {
    component: Option<T>,
    messages: Vec<T::Message>,
}

impl<T: ToComponent + Default + OnMessage> Default for CompHolder<T> {
    fn default() -> Self {
        Self {
            component: Some(T::default()),
            messages: vec![],
        }
    }
}

impl<T: ToComponent + OnMessage> CompHolder<T> {
    pub fn new(component: T) -> Self {
        Self {
            component: Some(component),
            messages: vec![],
        }
    }
    pub fn replace_component(&mut self, component: T) {
        self.component = Some(component);
    }

    pub fn send_message(&mut self, message: T::Message) {
        self.messages.push(message);
    }
}

#[doc(hidden)]
impl<T: ToComponent + OnMessage> CompHolder<T> {
    pub fn take(&mut self) -> Option<T> {
        self.component.take()
    }

    pub fn is_updated(&self) -> bool {
        self.component.is_some()
    }

    pub fn send_messages(&mut self, comp_struct: &mut T) {
        for msg in self.messages.drain(..) {
            comp_struct.on_message(msg)
        }
    }
}
