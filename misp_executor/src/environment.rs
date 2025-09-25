use std::collections::BTreeMap;

use misp_common::intern::StringId;
use misp_interner::Interner;
use misp_num::{Sign, decimal::Decimal};

use crate::{Function, RawNativeMispFunction, Value};

#[derive(Debug, Default, Clone)]
pub struct Scope {
    pub bindings: BTreeMap<StringId, Value>,
}

#[derive(Debug, Clone)]
pub struct Environment {
    next_function_id: usize,
    scopes: Vec<Scope>,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            next_function_id: 0,
            // This is the parent scope.
            scopes: vec![Scope::default()],
        }
    }
}

impl Environment {
    pub fn reset(&mut self) {
        self.next_function_id = 0;
        self.scopes = vec![Scope::default()];
    }

    pub fn load_constants(&mut self, interner: &mut Interner) {
        self.set_at_compile("math::pi", Value::Decimal(Decimal::PI), interner);
        self.set_at_compile("math::e", Value::Decimal(Decimal::E), interner);
        self.set_at_compile(
            "math::tau",
            Value::Decimal(Decimal::new(6283185307179586476, 18, Sign::Positive)),
            interner,
        );
        self.set_at_compile(
            "math::phi",
            Value::Decimal(Decimal::new(16180339887498948482, 19, Sign::Positive)),
            interner,
        );

        // Physics Constants
        self.set_at_compile(
            "physics::c",
            Value::Decimal(Decimal::new(299792458, 0, Sign::Positive)),
            interner,
        );
        self.set_at_compile(
            "physics::planck",
            Value::Decimal(Decimal::new(662607015, 42, Sign::Positive)),
            interner,
        );
        self.set_at_compile(
            "physics::gravity",
            Value::Decimal(Decimal::new(9810665, 6, Sign::Positive)),
            interner,
        );
        self.set_at_compile(
            "physics::G",
            Value::Decimal(Decimal::new(6673, 14, Sign::Positive)),
            interner,
        );

        // Chemistry Constants
        self.set_at_compile(
            "chemistry::avogadro",
            Value::Decimal(Decimal::new(6022142, -17, Sign::Positive)),
            interner,
        );
        self.set_at_compile(
            "chemistry::boltzmann",
            Value::Decimal(Decimal::new(1380651, 29, Sign::Positive)),
            interner,
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

    pub fn define_native_function(
        &mut self,
        name: &'static str,
        f: RawNativeMispFunction,
        interner: &mut Interner,
    ) {
        let function_id = self.next_function_id;
        self.next_function_id += 1;
        self.set_at_compile(
            name,
            Value::Function(Function::Native((function_id, f))),
            interner,
        );
    }

    pub fn set_prev(&mut self, value: Value, interner: &mut Interner) {
        let prev_id = interner.intern_string("prev".to_string()).unwrap();
        self.current_scope_mut().bindings.insert(prev_id, value);
    }

    fn set_at_compile(&mut self, name: &'static str, value: Value, interner: &mut Interner) {
        let name_id = interner.intern_string(name.to_string()).unwrap();
        self.current_scope_mut().bindings.insert(name_id, value);
    }

    pub fn set(&mut self, name: StringId, value: Value) {
        self.current_scope_mut().bindings.insert(name, value);
    }

    pub fn set_global(&mut self, name: StringId, value: Value) {
        self.global_scope_mut().bindings.insert(name, value);
    }

    pub fn get(&self, name: StringId) -> Option<&Value> {
        // Searches from the current scope up, trying to match the variable.
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.bindings.get(&name) {
                return Some(value);
            }
        }

        None
    }
}
