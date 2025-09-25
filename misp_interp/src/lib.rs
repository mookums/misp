use itertools::Itertools;
use misp_common::sexpr::SExpr;
use misp_executor::{Executor, Function, Value};
use misp_interner::Interner;
use misp_lexer::Lexer;
use misp_parser::Parser;

#[derive(Default)]
pub struct Misp {
    lexer: Lexer,
    parser: Parser,
    executor: Executor,
    interner: Interner,
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
        self.interner.reset();

        let input: &str = input.as_ref();
        let tokens = self.lexer.lex(input, &mut self.interner)?;
        self.parser.insert_tokens(tokens);
        let sexpr = self.parser.parse(&mut self.interner)?;

        let value = self.sexpr_to_value(&sexpr);

        self.executor.initalize_env(&mut self.interner);
        Ok(self.executor.execute(value)?)
    }

    pub fn eval_script(&mut self, input: impl AsRef<str>) -> Result<Vec<Value>, Error> {
        let input: &str = input.as_ref();
        let tokens = self.lexer.lex(input, &mut self.interner)?;
        self.parser.insert_tokens(tokens);
        let sexprs = self.parser.parse_multiple(&mut self.interner)?;
        self.executor.initalize_env(&mut self.interner);

        Ok(sexprs
            .into_iter()
            .map(|s| self.executor.execute(self.sexpr_to_value(&s)))
            .collect::<Result<Vec<Value>, misp_executor::Error>>()?)
    }

    fn sexpr_to_value(&self, sexpr: &SExpr) -> Value {
        match sexpr {
            SExpr::Atom(str) => Value::Atom(*str),
            SExpr::List(sexpr_ids) => Value::List(
                sexpr_ids
                    .iter()
                    .map(|seid| self.interner.get_sexpr(*seid).unwrap())
                    .map(|s| self.sexpr_to_value(s))
                    .collect(),
            ),
            SExpr::Decimal(decimal) => Value::Decimal(*decimal),
        }
    }

    pub fn print(&self, value: &Value) -> String {
        match value {
            Value::Atom(s) => {
                let str = self.interner.get_string(*s).unwrap();
                str.to_string()
            }
            Value::List(exprs) => {
                let items: Vec<String> = exprs.iter().map(|s| self.print(s)).collect();
                format!("({})", items.join(" "))
            }
            Value::Decimal(d) => d.to_scientific_notation(),
            Value::Function(func) => match func {
                Function::Native(_) => "<native>".to_string(),
                Function::Runtime(rt) => format!(
                    "<function> -> ({})",
                    rt.params
                        .iter()
                        .map(|sid| self.interner.get_string(*sid).unwrap())
                        .join(", ")
                ),
                Function::Lambda(l) => format!(
                    "<lambda> -> ({})",
                    l.params
                        .iter()
                        .map(|sid| self.interner.get_string(*sid).unwrap())
                        .join(", ")
                ),
            },
        }
    }
}
