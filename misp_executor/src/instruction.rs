use core::fmt::Display;

use compact_str::CompactString;

use crate::{Function, Value, cas::CasOperation};

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

#[macro_export]
macro_rules! variadic_op {
    ($e:ident, $op:tt) => {
        {
            const MAX_VARIADIC_ARGS: usize = 16;
            let mut values: [Decimal; MAX_VARIADIC_ARGS] = [const { Decimal::ZERO }; MAX_VARIADIC_ARGS];
            let Value::Decimal(arg_count) = $e.stack.pop().ok_or(Error::EmptyStack)? else {
                return Err(Error::InvalidType);
            };
            let arity = arg_count.to_u128() as usize;
            if arity == 0 {
                return Err(Error::InvalidType);
            }

            // Pop ALL arguments from stack
            for i in (0..arity).rev() {
                let thunk = $e.stack.pop().unwrap();
                values[i] = match thunk {
                    Value::Decimal(val) => val,
                    _ => return Err(Error::InvalidType),
                };
            }

            let mut acc = values[0];

            #[allow(clippy::assign_op_pattern)]
            for value in &values[1..arity] {
                acc = acc $op *value;
            }
            $e.stack.push(Value::Decimal(acc));
        }
    };
}

#[macro_export]
macro_rules! binary_comparison {
    ($e:expr, $op:tt) => {{
        let Value::Decimal(arity) = $e.stack.pop().ok_or(Error::EmptyStack)? else {
            return Err(Error::InvalidType);
        };

        debug_assert_eq!(arity.to_u128(), 2);

        let Value::Decimal(right) = $e.stack.pop().ok_or(Error::EmptyStack)? else {
            return Err(Error::InvalidType);
        };

        let Value::Decimal(left) = $e.stack.pop().ok_or(Error::EmptyStack)? else {
            return Err(Error::InvalidType);
        };

        let result = Decimal::from(left $op right);
        $e.stack.push(Value::Decimal(result));
    }};
}
