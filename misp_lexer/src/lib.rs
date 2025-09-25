#![no_std]
extern crate alloc;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use misp_common::token::Token;
use misp_interner::Interner;
use thiserror::Error;

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

    fn ident_token<'a>(
        &mut self,
        rest: &'a str,
        interner: &mut Interner,
    ) -> Option<(&'a str, Token)> {
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
        let id = interner.intern_string(token_str.to_string()).unwrap();
        Some((remaining, Token::Ident(id)))
    }

    fn token<'a>(&mut self, rest: &'a str, interner: &mut Interner) -> Option<(&'a str, Token)> {
        if let Some(literal) = self.literal_token(rest) {
            return Some(literal);
        }

        if let Some(pair) = self.single_token(rest) {
            return Some(pair);
        }

        self.ident_token(rest, interner)
    }

    fn reset(&mut self) {
        self.line = 0;
        self.column = 0;
    }

    pub fn lex(&mut self, input: &str, interner: &mut Interner) -> Result<Vec<Token>, Error> {
        self.reset();

        let mut tokens = Vec::new();

        for line in input.lines() {
            let mut rest = line;

            while !rest.is_empty() {
                rest = self.skip_whitespace(rest);
                if rest.is_empty() {
                    break;
                };

                match self.token(rest, interner) {
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
    use misp_num::decimal::Decimal;

    use super::*;

    macro_rules! assert_ident {
        ($token: expr, $interner: expr, $str: expr) => {
            if let Token::Ident(id) = $token {
                assert!($interner.get_string(id).is_some_and(|str| str == $str))
            } else {
                panic!()
            }
        };
    }

    #[test]
    fn test_basic_addition() {
        let mut interner = Interner::default();
        let mut lexer = Lexer::default();

        let input = "(+ 10 4)";
        let tokens = lexer.lex(input, &mut interner).unwrap();

        let mut kinds = tokens.into_iter();
        assert_eq!(kinds.next().unwrap(), Token::LeftParen);
        assert_ident!(kinds.next().unwrap(), interner, "+");
        assert_eq!(kinds.next().unwrap(), Token::Decimal(Decimal::from(10)));
        assert_eq!(kinds.next().unwrap(), Token::Decimal(Decimal::from(4)));
        assert_eq!(kinds.next().unwrap(), Token::RightParen);
    }

    #[test]
    fn test_compound_math() {
        let mut interner = Interner::default();
        let mut lexer = Lexer::default();

        let input = "(+ (* 10 15) 4)";
        let tokens = lexer.lex(input, &mut interner).unwrap();

        let mut kinds = tokens.into_iter();
        assert_eq!(kinds.next().unwrap(), Token::LeftParen);
        assert_ident!(kinds.next().unwrap(), interner, "+");
        assert_eq!(kinds.next().unwrap(), Token::LeftParen);
        assert_ident!(kinds.next().unwrap(), interner, "*");
        assert_eq!(kinds.next().unwrap(), Token::Decimal(Decimal::from(10)));
        assert_eq!(kinds.next().unwrap(), Token::Decimal(Decimal::from(15)));
        assert_eq!(kinds.next().unwrap(), Token::RightParen);
        assert_eq!(kinds.next().unwrap(), Token::Decimal(Decimal::from(4)));
        assert_eq!(kinds.next().unwrap(), Token::RightParen);
    }
}
