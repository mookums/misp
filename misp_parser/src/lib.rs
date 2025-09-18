use std::{collections::VecDeque, fmt::Display};

use bigdecimal::BigDecimal;
use misp_lexer::Token;
use num::{BigInt, BigRational};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unexpected Token")]
    UnexpectedToken,
}

#[derive(Debug, Clone)]
pub enum SExpr {
    Atom(String),
    List(Vec<SExpr>),

    Integer(BigInt),
    Decimal(BigDecimal),
    Rational(BigRational),
}

pub struct Parser {
    tokens: VecDeque<Token>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: tokens.into(),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<SExpr>, Error> {
        let mut exprs = Vec::new();

        while !self.is_at_end() {
            exprs.push(self.parse_expr()?);
        }

        Ok(exprs)
    }

    fn parse_expr(&mut self) -> Result<SExpr, Error> {
        let next = self.advance().unwrap();

        match next {
            Token::Ident(data) => Ok(SExpr::Atom(data)),
            Token::Integer(integer) => Ok(SExpr::Integer(integer)),
            Token::Rational(rational) => Ok(SExpr::Rational(rational)),
            Token::Decimal(decimal) => Ok(SExpr::Decimal(decimal)),
            Token::LeftParen => {
                let mut exprs = Vec::new();

                while self.peek().is_some_and(|p| p != &Token::RightParen) {
                    exprs.push(self.parse_expr()?);
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

impl Display for SExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SExpr::Atom(s) => write!(f, "{}", s),
            SExpr::List(exprs) => {
                let items: Vec<String> = exprs.iter().map(|e| e.to_string()).collect();
                write!(f, "({})", items.join(" "))
            }
            SExpr::Integer(n) => write!(f, "{n}"),
            SExpr::Decimal(d) => write!(f, "{d}",),
            SExpr::Rational(r) => write!(f, "{r}"),
        }
    }
}
