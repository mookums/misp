use std::collections::HashMap;

use crate::{Function, NativeMispFunction, Value};

#[derive(Debug, Default, Clone)]
pub struct Scope {
    pub bindings: HashMap<String, Value>,
}

#[derive(Debug, Clone, Default)]
pub struct Environment {
    scopes: Vec<Scope>,
}

impl Environment {
    pub fn current_scope(&self) -> &Scope {
        self.scopes.last().unwrap()
    }

    fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().unwrap()
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    pub fn push_given_scope(&mut self, scope: Scope) {
        self.scopes.push(scope);
    }

    pub fn pop_scope(&mut self) {
        debug_assert!(self.scopes.len() > 1);
        self.scopes.pop();
    }

    pub fn define_native_function(&mut self, name: impl ToString, f: NativeMispFunction) {
        self.current_scope_mut()
            .bindings
            .insert(name.to_string(), Value::Function(Function::Native(f)));
    }

    pub fn set_prev(&mut self, value: Value) {
        self.current_scope_mut()
            .bindings
            .insert("prev".to_string(), value);
    }

    pub fn set(&mut self, name: impl ToString, value: Value) {
        self.current_scope_mut()
            .bindings
            .insert(name.to_string(), value);
    }

    pub fn get(&self, name: impl AsRef<str>) -> Option<&Value> {
        // Searches from the current scope up, trying to match the variable.
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.bindings.get(name.as_ref()) {
                return Some(value);
            }
        }

        None
    }
}
