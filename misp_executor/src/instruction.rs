use core::fmt::Display;

use compact_str::CompactString;

use crate::{Function, Value, cas::CasOperation, environment::Scope};

#[derive(Debug, Clone)]
pub enum Instruction {
    Push(Value),
    Store(CompactString),
    Load(CompactString),
    TailCall(Function),
    Call(Function),
    Return,
    // Control Flow
    Jmp(usize),
    Jz(usize),
    // Arith Instructions
    Add,
    Sub,
    Mult,
    Div,
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
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

#[macro_export]
macro_rules! variadic_instruction {
    ($e: expr, $values: expr, $instr: ident) => {{
        let arity = ($values.len() - 1) as u64;

        for param in $values.into_iter().skip(1) {
            $e.compile_value(param.clone(), false)?;
        }

        $e.instructions
            .push(Instruction::Push(Value::Decimal(arity.into())));
        $e.instructions.push(Instruction::$instr);
    }};
}
