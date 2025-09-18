use std::collections::HashMap;

use misp_parser::SExpr;

use crate::{Error, Executor};

type NativeMispFunction = fn(&mut Executor, &[SExpr]) -> Result<SExpr, Error>;

#[derive(Clone)]
pub struct RuntimeMispFunction {
    pub params: Vec<String>,
    pub body: Box<SExpr>,
}

#[derive(Clone)]
pub enum Function {
    Native(NativeMispFunction),
    UserDefined(RuntimeMispFunction),
}

#[derive(Default)]
pub struct Scope {
    pub bindings: HashMap<String, SExpr>,
    pub functions: HashMap<String, Function>,
}

#[derive(Default)]
pub struct Environment {
    scopes: Vec<Scope>,
}

impl Environment {
    fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().unwrap()
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    pub fn pop_scope(&mut self) {
        assert!(self.scopes.len() > 1);
        self.scopes.pop();
    }

    pub fn define_native_function(&mut self, name: impl ToString, f: NativeMispFunction) {
        self.current_scope_mut()
            .functions
            .insert(name.to_string(), Function::Native(f));
    }

    pub fn set_prev(&mut self, value: SExpr) {
        self.current_scope_mut()
            .bindings
            .insert("prev".to_string(), value);
    }

    pub fn set_variable(&mut self, name: impl ToString, value: SExpr) {
        self.current_scope_mut()
            .bindings
            .insert(name.to_string(), value);
    }

    pub fn get_variable(&self, name: impl AsRef<str>) -> Option<&SExpr> {
        // Searches from the current scope up, trying to match the variable.
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.bindings.get(name.as_ref()) {
                return Some(value);
            }
        }

        None
    }

    pub fn set_function(&mut self, name: impl ToString, func: RuntimeMispFunction) {
        self.current_scope_mut()
            .functions
            .insert(name.to_string(), Function::UserDefined(func));
    }

    pub fn get_function(&self, name: impl AsRef<str>) -> Option<&Function> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.functions.get(name.as_ref()) {
                return Some(value);
            }
        }

        None
    }
}
