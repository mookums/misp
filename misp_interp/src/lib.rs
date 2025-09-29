use misp_executor::{
    Executor,
    instruction::Instruction,
    value::{Function, Value},
};
use misp_parser::Parser;

#[derive(Default)]
pub struct Misp {
    pub executor: Executor,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Parser error: {0}")]
    Parser(#[from] misp_parser::Error),
    #[error("Executor error: {0}")]
    Executor(#[from] misp_executor::Error),
}

impl Misp {
    pub fn eval(&mut self, input: &str) -> Result<Value, Error> {
        let mut parser = Parser::new(input);
        let sexpr = parser.parse()?;
        Ok(self.executor.execute(sexpr.into())?)
    }

    pub fn eval_script(&mut self, input: &str) -> Result<Vec<Value>, Error> {
        let mut parser = Parser::new(input);
        let sexprs = parser.parse_multiple()?;
        Ok(sexprs
            .into_iter()
            .map(|s| self.executor.execute(s.into()))
            .collect::<Result<Vec<Value>, misp_executor::Error>>()?)
    }

    pub fn compile(&mut self, input: &str) -> Result<(usize, Vec<Instruction>), Error> {
        let mut parser = Parser::new(input);
        let sexpr = parser.parse()?;

        Ok(self.executor.compile(sexpr.into())?)
    }

    pub fn compile_script(&mut self, input: &str) -> Result<(usize, Vec<Instruction>), Error> {
        let mut parser = Parser::new(input);
        let sexprs = parser.parse_multiple()?;

        for sexpr in &sexprs[0..sexprs.len() - 1] {
            self.executor.compile(sexpr.clone().into())?;
        }

        let last = sexprs.last().unwrap().clone();
        Ok(self.executor.compile(last.into())?)
    }

    pub fn print(value: &Value) -> String {
        match value {
            Value::Atom(s) => format!("'{}'", s),
            Value::Symbol(s) => s.to_string(),
            Value::List(exprs) => {
                let items: Vec<String> = exprs.iter().map(Self::print).collect();
                format!("({})", items.join(" "))
            }
            Value::Decimal(d) => d.to_scientific_notation(),
            // Value::Decimal(d) => format!("{d:?}"),
            Value::Function(func) => match func {
                Function::Runtime(rt) => format!("<function> -> ({})", rt.params.join(", ")),
                Function::Lambda(l) => format!("<lambda> -> ({})", l.params.join(", ")),
            },
        }
    }
}
