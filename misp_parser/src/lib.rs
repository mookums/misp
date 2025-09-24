#![no_std]

use heapless::{Deque, Vec};
use misp_common::{intern::SExprId, sexpr::SExpr, token::Token};
use misp_interner::Interner;

use misp_num::decimal::Decimal;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unexpected Token")]
    UnexpectedToken,
}

#[derive(Debug, Clone, Default)]
pub struct Parser<
    const MAX_STR: usize,
    const MAX_TOKENS: usize,
    const MAX_LIST: usize,
    const MAX_INTERN: usize,
> {
    tokens: Deque<Token<MAX_STR>, MAX_TOKENS>,
}

impl<const MAX_STR: usize, const MAX_TOKENS: usize, const MAX_LIST: usize, const MAX_INTERN: usize>
    Parser<MAX_STR, MAX_TOKENS, MAX_LIST, MAX_INTERN>
{
    pub fn new(
        tokens: Vec<Token<MAX_STR>, MAX_STR>,
    ) -> Parser<MAX_STR, MAX_TOKENS, MAX_LIST, MAX_INTERN> {
        let mut deque = Deque::new();

        // Convert Vec to Deque
        for token in tokens {
            deque.push_back(token).unwrap();
        }

        Self { tokens: deque }
    }

    pub fn insert_tokens(&mut self, tokens: Vec<Token<MAX_STR>, MAX_TOKENS>) {
        for token in tokens {
            self.tokens.push_back(token).unwrap();
        }
    }

    pub fn parse(
        &mut self,
        interner: &mut Interner<MAX_STR, MAX_LIST, MAX_INTERN>,
    ) -> Result<SExprId, Error> {
        self.parse_expr(interner)
    }

    fn parse_expr(
        &mut self,
        interner: &mut Interner<MAX_STR, MAX_LIST, MAX_INTERN>,
    ) -> Result<SExprId, Error> {
        let next = self.advance().unwrap();

        let expr = match next {
            Token::Ident(data) => SExpr::Atom(data),
            Token::Decimal(decimal) => SExpr::Decimal(decimal),
            Token::LeftParen => {
                let mut exprs = Vec::new();

                while self.peek().is_some_and(|p| p != &Token::RightParen) {
                    let id = self.parse_expr(interner)?;
                    exprs.push(id).unwrap();
                }

                match self.advance() {
                    Some(Token::RightParen) => {}
                    _ => return Err(Error::UnexpectedToken),
                }

                SExpr::List(exprs)
            }
            _ => todo!(),
        };

        Ok(interner.intern_sexpr(expr).unwrap())
    }

    fn peek(&self) -> Option<&Token<MAX_STR>> {
        self.tokens.front()
    }

    fn advance(&mut self) -> Option<Token<MAX_STR>> {
        self.tokens.pop_front()
    }
}
