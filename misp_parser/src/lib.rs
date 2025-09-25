#![no_std]
extern crate alloc;

use alloc::{collections::vec_deque::VecDeque, vec::Vec};
use misp_common::{sexpr::SExpr, token::Token};
use misp_interner::Interner;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unexpected Token")]
    UnexpectedToken,
}

#[derive(Debug, Clone, Default)]
pub struct Parser {
    tokens: VecDeque<Token>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: tokens.into(),
        }
    }

    pub fn insert_tokens(&mut self, tokens: Vec<Token>) {
        self.tokens = tokens.into();
    }

    pub fn parse(&mut self, interner: &mut Interner) -> Result<SExpr, Error> {
        self.parse_expr(interner)
    }

    pub fn parse_multiple(&mut self, interner: &mut Interner) -> Result<Vec<SExpr>, Error> {
        let mut exprs = Vec::new();

        while !self.is_at_end() {
            exprs.push(self.parse_expr(interner)?);
        }

        Ok(exprs)
    }

    fn parse_expr(&mut self, interner: &mut Interner) -> Result<SExpr, Error> {
        let next = self.advance().unwrap();

        match next {
            Token::Ident(data) => Ok(SExpr::Atom(data)),
            Token::Decimal(decimal) => Ok(SExpr::Decimal(decimal)),
            Token::LeftParen => {
                let mut exprs = Vec::new();

                while self.peek().is_some_and(|p| p != &Token::RightParen) {
                    let expr = self.parse_expr(interner)?;
                    let id = interner.intern_sexpr(expr).unwrap();
                    exprs.push(id);
                }

                match self.advance() {
                    Some(Token::RightParen) => {}
                    _ => return Err(Error::UnexpectedToken),
                }

                Ok(SExpr::List(exprs))
            }
            _ => todo!(),
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.front()
    }

    fn advance(&mut self) -> Option<Token> {
        self.tokens.pop_front()
    }

    fn is_at_end(&self) -> bool {
        self.peek().is_none()
    }
}
