use misp_executor::{Executor, Function, Value};
use misp_lexer::Lexer;
use misp_parser::Parser;

#[derive(Debug, Clone, Default)]
pub struct Misp {
    lexer: Lexer,
    parser: Parser,
    executor: Executor,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Lexer error: {0}")]
    Lexer(#[from] misp_lexer::Error),
    #[error("Parser error: {0}")]
    Parser(#[from] misp_parser::Error),
    #[error("Executor error: {0}")]
    Executor(#[from] misp_executor::Error),
}

impl Misp {
    pub fn eval(&mut self, input: impl AsRef<str>) -> Result<Value, Error> {
        let input: &str = input.as_ref();
        let tokens = self.lexer.lex(input)?;
        self.parser.insert_tokens(tokens);
        let sexpr = self.parser.parse()?;
        Ok(self.executor.execute(&sexpr)?)
    }

    pub fn eval_to_string(&mut self, input: impl AsRef<str>) -> Result<String, Error> {
        let value = self.eval(input)?;
        Ok(Self::print(&value))
    }

    pub fn print(value: &Value) -> String {
        match value {
            Value::Atom(s) => s.to_string(),
            Value::List(exprs) => {
                let items: Vec<String> = exprs.iter().map(Self::print).collect();
                format!("({})", items.join(" "))
            }
            Value::Decimal(d) => format!("{d}"),
            // Value::Decimal(d) => format!("{d:?}"),
            Value::Function(func) => match func {
                Function::Native(_) => "<native>".to_string(),
                Function::Runtime(rt) => format!("<function> -> ({})", rt.params.join(", ")),
                Function::Lambda(l) => format!("<lambda> -> ({})", l.params.join(", ")),
            },
        }
    }
}
