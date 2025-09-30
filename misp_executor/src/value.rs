use core::hash::Hash;

use alloc::{boxed::Box, rc::Rc, vec::Vec};
use compact_str::CompactString;
use misp_num::decimal::Decimal;
use misp_parser::SExpr;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Lambda {
    pub params: Vec<CompactString>,
    pub body: Box<Value>,
}

#[derive(Debug, Clone)]
pub struct RuntimeMispFunction {
    pub id: usize,
    pub params: Rc<Vec<CompactString>>,
    pub body: Rc<Value>,
}

impl PartialEq for RuntimeMispFunction {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for RuntimeMispFunction {}

impl Hash for RuntimeMispFunction {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Ord for RuntimeMispFunction {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for RuntimeMispFunction {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Function {
    Runtime(RuntimeMispFunction),
    Lambda(Lambda),
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum Value {
    Atom(CompactString),
    Symbol(CompactString),
    List(Vec<Value>),
    Decimal(Decimal),
    Function(Function),
}

impl From<SExpr> for Value {
    fn from(value: SExpr) -> Self {
        match value {
            SExpr::Atom(str) => Value::Atom(str),
            SExpr::List(sexprs) => Value::List(sexprs.into_iter().map(|e| e.into()).collect()),
            SExpr::Decimal(d) => Value::Decimal(d),
        }
    }
}
