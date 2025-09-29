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

#[macro_export]
macro_rules! unary_operation {
    ($e: expr, $values: expr, $op: ident) => {{
        let arity = ($values.len() - 1) as u64;
        if arity != 1 {
            panic!("arity mismatch on unary op")
        }

        for param in $values.into_iter().skip(1) {
            $e.compile_value(param.clone(), CallKind::Normal)?;
        }

        $e.instructions
            .push(Instruction::Push(Value::Decimal(arity.into())));

        $e.instructions
            .push(Instruction::Operation(Operation::Unary(
                UnaryOperation::$op,
            )));
    }};
}

#[macro_export]
macro_rules! binary_operation {
    ($e: expr, $values: expr, $op: ident) => {{
        let arity = ($values.len() - 1) as u64;
        if arity != 2 {
            panic!("arity mismatch on binary op")
        }

        for param in $values.into_iter().skip(1) {
            $e.compile_value(param.clone(), CallKind::Normal)?;
        }

        $e.instructions
            .push(Instruction::Push(Value::Decimal(arity.into())));

        $e.instructions
            .push(Instruction::Operation(Operation::Binary(
                BinaryOperation::$op,
            )));
    }};
}

#[macro_export]
macro_rules! variadic_operation {
    ($e: expr, $values: expr, $op: ident) => {{
        let arity = ($values.len() - 1) as u64;

        for param in $values.into_iter().skip(1) {
            $e.compile_value(param.clone(), CallKind::Normal)?;
        }

        $e.instructions
            .push(Instruction::Push(Value::Decimal(arity.into())));

        $e.instructions
            .push(Instruction::Operation(Operation::Variadic(
                VariadicOperation::$op,
            )));
    }};
}
