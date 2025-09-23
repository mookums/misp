use std::collections::HashMap;

use misp_num::{Sign, decimal::Decimal};

use crate::{Function, NativeMispFunction, Value};

#[derive(Debug, Default, Clone)]
pub struct Scope {
    pub bindings: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct Environment {
    scopes: Vec<Scope>,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            // This is the parent scope.
            scopes: vec![Scope::default()],
        }
    }
}

impl Environment {
    pub fn load_constants(&mut self) {
        self.set("math::pi", Value::Decimal(Decimal::PI));
        self.set("math::e", Value::Decimal(Decimal::E));
        self.set(
            "math::tau",
            Value::Decimal(Decimal::new(6283185307179586476, 18, Sign::Positive)),
        );
        self.set(
            "math::phi",
            Value::Decimal(Decimal::new(16180339887498948482, 19, Sign::Positive)),
        );

        // Physics Constants
        self.set(
            "physics::c",
            Value::Decimal(Decimal::new(299792458, 0, Sign::Positive)),
        );
        self.set(
            "physics::planck",
            Value::Decimal(Decimal::new(662607015, 42, Sign::Positive)),
        );
        self.set(
            "physics::gravity",
            Value::Decimal(Decimal::new(9810665, 6, Sign::Positive)),
        );
        self.set(
            "physics::G",
            Value::Decimal(Decimal::new(6673, 14, Sign::Positive)),
        );

        // Chemistry Constants
        self.set(
            "chemistry::avogadro",
            Value::Decimal(Decimal::new(6022142, -17, Sign::Positive)),
        );
        self.set(
            "chemistry::boltzmann",
            Value::Decimal(Decimal::new(1380651, 29, Sign::Positive)),
        );
    }

    pub fn global_scope_mut(&mut self) -> &mut Scope {
        self.scopes.first_mut().unwrap()
    }

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

    pub fn set_global(&mut self, name: impl ToString, value: Value) {
        self.global_scope_mut()
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
