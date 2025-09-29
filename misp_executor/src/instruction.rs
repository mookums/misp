use core::fmt::Display;

use compact_str::CompactString;

use crate::{Function, Value, cas::CasOperation, operation::Operation};

#[derive(Debug, Clone)]
pub enum Instruction {
    Push(Value),
    Store(CompactString),
    Load(CompactString),
    // Directly call a function.
    Call(Function),
    // Indirectly call a function from the stack.
    CallIndirect,
    TailCall(Function),
    TailCallIndirect,
    Return,
    // Control Flow
    Jmp(usize),
    Jz(usize),
    // Arith Instructions
    Operation(Operation),
    // Cas
    Cas(CasOperation),
    Placeholder,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Instruction::Push(value) => write!(f, "Push({:?})", value),
            Instruction::Store(str) => write!(f, "Store({str})"),
            Instruction::Load(str) => write!(f, "Load({str})"),
            Instruction::Call(func) => write!(f, "Call({func:?})"),
            _ => write!(f, "{self:?}"),
        }
    }
}
