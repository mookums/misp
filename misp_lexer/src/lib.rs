use std::str::FromStr;

use bigdecimal::BigDecimal;
use num::{BigInt, BigRational};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    LeftParen,
    RightParen,
    Ident(String),

    Integer(BigInt),
    Decimal(BigDecimal),
    Rational(BigRational),
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

#[derive(Default)]
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
        let first_char = rest.chars().next()?;

        match first_char {
            c if c.is_ascii_digit() || c == '-' => {
                let chars = rest.char_indices();
                let mut end_pos = 0;
                let mut has_dot = false;
                let mut is_first = true;

                for (pos, ch) in chars {
                    match ch {
                        '0'..='9' => {
                            end_pos = pos + ch.len_utf8();
                            is_first = false;
                        }
                        '.' if !has_dot && !is_first => {
                            has_dot = true;
                            end_pos = pos + ch.len_utf8();
                        }
                        '-' if is_first => {
                            end_pos = pos + ch.len_utf8();
                            is_first = false;
                        }
                        _ if ch.is_whitespace() || "()".contains(ch) => break,
                        _ => return None, // Invalid character in number
                    }
                }

                if end_pos == 0 || (end_pos == 1 && first_char == '-') {
                    return None;
                }

                let token_str = &rest[..end_pos];
                let remaining = &rest[end_pos..];

                let token = if has_dot {
                    let decimal = BigDecimal::from_str(token_str).unwrap();
                    Token::Decimal(decimal)
                } else {
                    let integer = BigInt::from_str(token_str).unwrap();
                    Token::Integer(integer)
                };

                self.column += token_str.chars().count();

                Some((remaining, token))
            }

            _ => None,
        }
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

    pub fn lex(mut self, input: &str) -> Result<Vec<Token>, Error> {
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
        let lexer = Lexer::default();
        let input = "(+ 10 4)";
        let tokens = lexer.lex(input).unwrap();

        let mut kinds = tokens.iter();
        assert_eq!(kinds.next().unwrap(), &Token::LeftParen);
        assert_eq!(kinds.next().unwrap(), &Token::Ident("+".to_string()));
        assert_eq!(kinds.next().unwrap(), &Token::Integer(BigInt::from(10)));
        assert_eq!(kinds.next().unwrap(), &Token::Integer(BigInt::from(4)));
        assert_eq!(kinds.next().unwrap(), &Token::RightParen);
    }

    #[test]
    fn test_compound_math() {
        let lexer = Lexer::default();
        let input = "(+ (* 10 15) 4)";
        let tokens = lexer.lex(input).unwrap();

        let mut kinds = tokens.iter();
        assert_eq!(kinds.next().unwrap(), &Token::LeftParen);
        assert_eq!(kinds.next().unwrap(), &Token::Ident("+".to_string()));
        assert_eq!(kinds.next().unwrap(), &Token::LeftParen);
        assert_eq!(kinds.next().unwrap(), &Token::Ident("*".to_string()));
        assert_eq!(kinds.next().unwrap(), &Token::Integer(BigInt::from(10)));
        assert_eq!(kinds.next().unwrap(), &Token::Integer(BigInt::from(15)));
        assert_eq!(kinds.next().unwrap(), &Token::RightParen);
        assert_eq!(kinds.next().unwrap(), &Token::Integer(BigInt::from(4)));
        assert_eq!(kinds.next().unwrap(), &Token::RightParen);
    }
}
