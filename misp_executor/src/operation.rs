#[derive(Debug, Clone, Copy)]
pub enum VariadicOperation {
    Add,
    Sub,
    Mult,
    Div,
}

#[derive(Debug, Clone, Copy)]
pub enum BinaryOperation {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
    Pow,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOperation {
    Sqrt,
    Abs,
}

#[derive(Debug, Clone, Copy)]
pub enum Operation {
    Variadic(VariadicOperation),
    Binary(BinaryOperation),
    Unary(UnaryOperation),
}
