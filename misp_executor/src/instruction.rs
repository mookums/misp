use core::fmt::Display;

use compact_str::CompactString;

use crate::{Function, Value, environment::Scope};

#[derive(Debug, Clone)]
pub enum Instruction {
    Push(Value),
    Store(CompactString),
    Load(CompactString),
    Call(Function),
    Return,
    // CallNative(usize),
    PushScope,
    PushDefinedScope(Scope),
    PopScope,
    // MemoCheck {
    //     id: usize,
    //     params: Rc<Vec<CompactString>>,
    // },
    // MemoStore {
    //     id: usize,
    //     params: Rc<Vec<CompactString>>,
    // },
}

impl Display for Instruction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Instruction::Push(value) => write!(f, "Push({:?})", value),
            Instruction::Store(str) => write!(f, "Store({str})"),
            Instruction::Load(str) => write!(f, "Load({str})"),
            Instruction::Call(func) => write!(f, "Call({func:?})"),
            Instruction::PushScope => write!(f, "PushScope"),
            Instruction::PushDefinedScope(_) => write!(f, "PushDefinedScope"),
            Instruction::PopScope => write!(f, "PopScope"),
            _ => write!(f, "{self:?}"),
        }
    }
}
