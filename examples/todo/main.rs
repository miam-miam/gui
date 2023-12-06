mod todo {
    use super::todos::Message;

    #[derive(Component)]
    pub struct Todo {
        name: Updateable<String>,
        completed: Updateable<bool>,
        id: usize,
    }

    impl Todo {
        pub fn new(todo_name: &str, task_id: usize) -> Todo {
            Todo {
                state: Updateable::new(gen::state::Default()),
                name: Updateable::new(todo_name.into()),
                completed: Updateable::new(false),
                id: task_id,
            }
        }
    }

    impl CheckboxAction for gen::CompletedBox {
        fn on_toggle(&mut self, is_on: bool, _e: MouseEvent) {
            self.completed.as_mut() = is_on;
        }
    }

    impl ButtonAction for gen::EditBtn {
        fn on_press(&mut self, _e: MouseEvent) {
            self.state.as_mut() = gen::state::Edit;
        }
    }

    impl ButtonAction for gen::DeleteBtn {
        fn on_press(&mut self, e: MouseEvent) {
            e.ctx().send_message(Message::RemoveTask(self.id));
        }
    }

    impl TextInputAction for gen::NameInput {
        fn on_submit(&mut self, value: &str) {
            self.name.as_mut() = value.into();
            self.state.as_mut() = gen::state::View;
        }
    }
}

// Must be in separate modules due to #[derive(Component)] generating a mod gen.
mod todos {
    use super::todo::Todo;

    #[derive(Component, Default)]
    pub struct Todos {
        tasks: Updateable<Vec<Todo>>,
    }

    impl Update for gen::sum_tasks {
        fn is_updated(&self) -> bool {
            self.tasks.is_updated()
        }

        fn value(&self) -> u32 {
            &self.tasks.value().iter()
                .filter(|t| !t.completed)
                .sum()
        }
    }


    pub enum Message {
        RemoveTask(usize)
    }

    impl OnMessage<Message> for Todos {
        fn on_receive(&mut self, message: Message) {
            match message {
                Message::RemoveTask(id) => {
                    self.tasks.as_mut().remove(id);
                }
            }
        }
    }
    impl TextInputAction for gen::TodoDescription {
        fn on_submit(&mut self, value: &mut String) {
            self.tasks.as_mut()
                .push(Todo::new(&value, tasks.as_ref().len()));
        }
    }

    impl ButtonAction for gen::All {
        fn on_press(&mut self, _e: MouseEvent) {
            self.state.as_mut() = gen::state::All;
        }
    }

    impl ButtonAction for gen::Active {
        fn on_press(&mut self, _e: MouseEvent) {
            self.state.as_mut() = gen::state::Active;
        }
    }

    impl ButtonAction for gen::Completed {
        fn on_press(&mut self, _e: MouseEvent) {
            self.state.as_mut() = gen::state::Completed;
        }
    }
}
fn main() {
    gui::run("Todo App", todos::Todos::default())
}