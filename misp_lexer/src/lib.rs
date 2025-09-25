#![no_std]
extern crate alloc;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use misp_num::decimal::Decimal;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    LeftParen,
    RightParen,
    Ident(String),

    Decimal(Decimal),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unrecognized Token: ({value:?}) at ({line:?}, {column:?})")]
    UnrecognizedToken {
        value: String,
        line: usize,
        column: usize,
    },
}

#[derive(Debug, Clone, Default)]
pub struct Lexer {
    line: usize,
    column: usize,
}

impl Lexer {
    fn skip_whitespace<'a>(&mut self, rest: &'a str) -> &'a str {
        let mut remaining = rest;

        loop {
            let c = remaining.chars().next();
            if let Some(c) = c
                && c.is_whitespace()
            {
                self.column += 1;
                remaining = &remaining[c.len_utf8()..];
            } else {
                break;
            }
        }

        remaining
    }

    fn single_token<'a>(&mut self, rest: &'a str) -> Option<(&'a str, Token)> {
        let char = rest.chars().next()?;
        let remaining = &rest[char.len_utf8()..];

        let token = match char {
            '(' => Token::LeftParen,
            ')' => Token::RightParen,
            _ => return None,
        };

        self.column += 1;

        Some((remaining, token))
    }

    fn literal_token<'a>(&mut self, rest: &'a str) -> Option<(&'a str, Token)> {
        let chars = rest.char_indices();
        let mut end_pos = rest.len();

        for (pos, ch) in chars {
            if ch.is_whitespace() || "()".contains(ch) {
                break;
            }

            end_pos = pos + ch.len_utf8();
        }

        if end_pos == 0 {
            return None;
        }

        let token_str = &rest[..end_pos];
        let remaining = &rest[end_pos..];

        if !token_str
            .chars()
            .all(|c| c.is_ascii_digit() || c == '-' || c == '.' || c == '/')
        {
            return None;
        }

        let token = Token::Decimal(token_str.parse().ok()?);
        self.column += token_str.chars().count();
        Some((remaining, token))
    }

    fn ident_token<'a>(&mut self, rest: &'a str) -> Option<(&'a str, Token)> {
        let chars = rest.char_indices();
        let mut end_pos = 0;

        for (pos, ch) in chars {
            if ch.is_whitespace() || "()".contains(ch) {
                break;
            }

            end_pos = pos + ch.len_utf8();
        }

        if end_pos == 0 {
            return None;
        }

        let token_str = &rest[..end_pos];
        let remaining = &rest[end_pos..];

        self.column += token_str.chars().count();

        Some((remaining, Token::Ident(token_str.to_string())))
    }

    fn token<'a>(&mut self, rest: &'a str) -> Option<(&'a str, Token)> {
        if let Some(literal) = self.literal_token(rest) {
            return Some(literal);
        }

        if let Some(pair) = self.single_token(rest) {
            return Some(pair);
        }

        self.ident_token(rest)
    }

    fn reset(&mut self) {
        self.line = 0;
        self.column = 0;
    }

    pub fn lex(&mut self, input: &str) -> Result<Vec<Token>, Error> {
        self.reset();

        let mut tokens = Vec::new();

        for line in input.lines() {
            let mut rest = line;

            while !rest.is_empty() {
                rest = self.skip_whitespace(rest);
                if rest.is_empty() {
                    break;
                };

                match self.token(rest) {
                    Some(pair) => {
                        rest = pair.0;
                        tokens.push(pair.1)
                    }
                    None => {
                        return Err(Error::UnrecognizedToken {
                            value: rest.to_string(),
                            line: self.line,
                            column: self.column,
                        });
                    }
                }
            }

            self.line += 1;
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_addition() {
        let mut lexer = Lexer::default();
        let input = "(+ 10 4)";
        let tokens = lexer.lex(input).unwrap();

        let mut kinds = tokens.iter();
        assert_eq!(kinds.next().unwrap(), &Token::LeftParen);
        assert_eq!(kinds.next().unwrap(), &Token::Ident("+".to_string()));
        assert_eq!(kinds.next().unwrap(), &Token::Decimal(Decimal::from(10)));
        assert_eq!(kinds.next().unwrap(), &Token::Decimal(Decimal::from(4)));
        assert_eq!(kinds.next().unwrap(), &Token::RightParen);
    }

    #[test]
    fn test_compound_math() {
        let mut lexer = Lexer::default();
        let input = "(+ (* 10 15) 4)";
        let tokens = lexer.lex(input).unwrap();

        let mut kinds = tokens.iter();
        assert_eq!(kinds.next().unwrap(), &Token::LeftParen);
        assert_eq!(kinds.next().unwrap(), &Token::Ident("+".to_string()));
        assert_eq!(kinds.next().unwrap(), &Token::LeftParen);
        assert_eq!(kinds.next().unwrap(), &Token::Ident("*".to_string()));
        assert_eq!(kinds.next().unwrap(), &Token::Decimal(Decimal::from(10)));
        assert_eq!(kinds.next().unwrap(), &Token::Decimal(Decimal::from(15)));
        assert_eq!(kinds.next().unwrap(), &Token::RightParen);
        assert_eq!(kinds.next().unwrap(), &Token::Decimal(Decimal::from(4)));
        assert_eq!(kinds.next().unwrap(), &Token::RightParen);
    }
}
