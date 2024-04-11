use crate::OnMessage;

#[derive(Clone, Debug)]
pub struct CompHolder<T: OnMessage> {
    component: Option<T>,
    messages: Vec<T::Message>,
}

impl<T: Default + OnMessage> Default for CompHolder<T> {
    fn default() -> Self {
        Self {
            component: Some(T::default()),
            messages: vec![],
        }
    }
}

impl<T: OnMessage> CompHolder<T> {
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
impl<T: OnMessage> CompHolder<T> {
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

#[cfg(test)]
mod test {
    use crate::OnMessage;

    #[test]
    fn test_message_sending() {
        #[derive(Default)]
        struct TestComponent {
            value: u32,
        }

        #[derive(Clone)]
        enum Message {
            Add(u32),
        }

        impl OnMessage for TestComponent {
            type Message = Message;

            fn on_message(&mut self, message: Message) {
                match message {
                    Message::Add(v) => self.value += v,
                }
            }
        }

        let mut comp_holder = super::CompHolder::new(TestComponent::default());
        comp_holder.send_message(Message::Add(5));
        comp_holder.send_message(Message::Add(10));
        let mut comp = comp_holder.take().unwrap();
        comp_holder.send_messages(&mut comp);
        assert_eq!(comp.value, 15);
    }
}
