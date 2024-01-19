use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Default)]
pub struct Updateable<T> {
    updated: AtomicBool,
    value: T,
}

impl<T> Updateable<T> {
    pub fn new(value: T) -> Self {
        Updateable {
            updated: AtomicBool::new(false),
            value,
        }
    }
    pub fn is_updated(&self) -> bool {
        self.updated.swap(false, Ordering::AcqRel)
    }

    pub fn set_value(&mut self, val: T) {
        self.value = val;
        self.updated.store(true, Ordering::Release);
    }

    pub fn get_value(&self) -> &T {
        &self.value
    }
}

impl<T: Clone> Updateable<T> {
    pub fn value(&self) -> T {
        self.value.clone()
    }
}
