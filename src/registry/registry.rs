use crate::registry::Type;
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Default)]
pub struct Registry {
    types: HashMap<String, Type>,
}

impl Deref for Registry {
    type Target = HashMap<String, Type>;

    fn deref(&self) -> &Self::Target {
        &self.types
    }
}

impl Registry {
    pub fn create_type<F: FnMut(&mut Registry) -> Type>(&mut self, name: &str, mut f: F) -> String {
        if !self.types.contains_key(name) {
            let ty = f(self);
            self.types.insert(name.to_string(), ty);
        }
        name.to_string()
    }
}
