use crate::schema::Type;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Default)]
pub struct Registry {
    types: HashMap<String, Type>,
}

impl Registry {
    pub fn create_type<F: FnMut(&mut Registry) -> Type>(&mut self, name: &str, mut f: F) -> String {
        if !self.types.contains_key(name) {
            let ty = f(self);
            self.types.insert(name.to_string(), ty);
        }
        name.to_string()
    }

    pub fn get_type(&self, name: &str) -> Option<&Type> {
        self.types.get(name)
    }
}
