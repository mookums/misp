use crate::value::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariadicOperation {
    Add,
    Sub,
    Mult,
    Div,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperation {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
    Pow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperation {
    Sqrt,
    Abs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Variadic(VariadicOperation),
    Binary(BinaryOperation),
    Unary(UnaryOperation),
}

pub fn parse_operation(s: &str) -> Option<Operation> {
    match s {
        "+" => Some(Operation::Variadic(VariadicOperation::Add)),
        "-" => Some(Operation::Variadic(VariadicOperation::Sub)),
        "*" => Some(Operation::Variadic(VariadicOperation::Mult)),
        "/" => Some(Operation::Variadic(VariadicOperation::Div)),
        "==" => Some(Operation::Binary(BinaryOperation::Eq)),
        ">" => Some(Operation::Binary(BinaryOperation::Gt)),
        ">=" => Some(Operation::Binary(BinaryOperation::Gte)),
        "<" => Some(Operation::Binary(BinaryOperation::Lt)),
        "<=" => Some(Operation::Binary(BinaryOperation::Lte)),
        "**" => Some(Operation::Binary(BinaryOperation::Pow)),
        "sqrt" => Some(Operation::Unary(UnaryOperation::Sqrt)),
        "abs" => Some(Operation::Unary(UnaryOperation::Abs)),
        _ => None,
    }
}

impl Operation {
    pub fn is_associative(&self) -> bool {
        matches!(
            self,
            Operation::Variadic(VariadicOperation::Add)
                | Operation::Variadic(VariadicOperation::Mult)
        )
    }

    pub fn to_atom_value(self) -> Value {
        let str = match self {
            Operation::Variadic(variadic) => match variadic {
                VariadicOperation::Add => "+",
                VariadicOperation::Sub => "-",
                VariadicOperation::Mult => "*",
                VariadicOperation::Div => "/",
            },
            Operation::Binary(binary) => match binary {
                BinaryOperation::Eq => "==",
                BinaryOperation::Neq => "!=",
                BinaryOperation::Gt => ">",
                BinaryOperation::Gte => ">=",
                BinaryOperation::Lt => "<",
                BinaryOperation::Lte => "<=",
                BinaryOperation::Pow => "**",
            },
            Operation::Unary(unary) => match unary {
                UnaryOperation::Sqrt => "sqrt",
                UnaryOperation::Abs => "abs",
            },
        };

        Value::Atom(str.into())
    }
}
