#![no_std]

use core::str::FromStr;

use misp_common::token::Token;
use misp_interner::Interner;
use thiserror::Error;

use heapless::{String, Vec};

#[derive(Debug, Error)]
pub enum Error<const MAX_STR: usize, const MAX_TOKENS: usize> {
    #[error("Unrecognized Token: ({value:?}) at ({line:?}, {column:?})")]
    UnrecognizedToken {
        value: String<MAX_STR>,
        line: usize,
        column: usize,
    },
    #[error("String too long, passed limit of {MAX_STR}")]
    StringTooLong,
    #[error("Too many tokens, passed limit of {MAX_TOKENS}")]
    TooManyTokens,
    #[error("Interner: {0}")]
    Interner(#[from] misp_interner::Error),
}

#[derive(Debug, Clone, Default)]
pub struct Lexer<
    const MAX_STR: usize,
    const MAX_TOKENS: usize,
    const MAX_LIST: usize,
    const MAX_INTERN: usize,
> {
    line: usize,
    column: usize,
}

impl<const MAX_STR: usize, const MAX_TOKENS: usize, const MAX_LIST: usize, const MAX_INTERN: usize>
    Lexer<MAX_STR, MAX_TOKENS, MAX_LIST, MAX_INTERN>
{
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

    fn single_token<'a>(&mut self, rest: &'a str) -> Option<(&'a str, Token<MAX_STR>)> {
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

    fn literal_token<'a>(&mut self, rest: &'a str) -> Option<(&'a str, Token<MAX_STR>)> {
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
        interner: &mut Interner<MAX_STR, MAX_LIST, MAX_INTERN>,
    ) -> Result<Option<(&'a str, Token<MAX_STR>)>, Error<MAX_STR, MAX_TOKENS>> {
        let chars = rest.char_indices();
        let mut end_pos = 0;

        for (pos, ch) in chars {
            if ch.is_whitespace() || "()".contains(ch) {
                break;
            }

            end_pos = pos + ch.len_utf8();
        }

        if end_pos == 0 {
            return Ok(None);
        }

        let token_str = &rest[..end_pos];
        let remaining = &rest[end_pos..];

        let id = interner.intern_string(token_str)?;
        self.column += token_str.chars().count();
        Ok(Some((remaining, Token::Ident(id))))
    }

    fn token<'a>(
        &mut self,
        rest: &'a str,
        interner: &mut Interner<MAX_STR, MAX_LIST, MAX_INTERN>,
    ) -> Result<Option<(&'a str, Token<MAX_STR>)>, Error<MAX_STR, MAX_TOKENS>> {
        if let Some(literal) = self.literal_token(rest) {
            return Ok(Some(literal));
        }

        if let Some(pair) = self.single_token(rest) {
            return Ok(Some(pair));
        }

        self.ident_token(rest, interner)
    }

    fn reset(&mut self) {
        self.line = 0;
        self.column = 0;
    }

    pub fn lex(
        &mut self,
        input: &str,
        interner: &mut Interner<MAX_STR, MAX_LIST, MAX_INTERN>,
    ) -> Result<Vec<Token<MAX_STR>, MAX_TOKENS>, Error<MAX_STR, MAX_TOKENS>> {
        self.reset();

        let mut tokens = Vec::new();

        for line in input.lines() {
            let mut rest = line;

            while !rest.is_empty() {
                rest = self.skip_whitespace(rest);
                if rest.is_empty() {
                    break;
                };

                match self.token(rest, interner)? {
                    Some(pair) => {
                        rest = pair.0;
                        tokens.push(pair.1).map_err(|_| Error::TooManyTokens)?
                    }
                    None => {
                        return Err(Error::UnrecognizedToken {
                            value: String::from_str(rest).map_err(|_| Error::StringTooLong)?,
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

    const MAX_STR: usize = 32;
    const MAX_LIST: usize = 0;
    const MAX_INTERN: usize = 32;

    type DefaultLexer = Lexer<MAX_STR, 1024, MAX_LIST, MAX_INTERN>;
    type DefaultInterner = Interner<MAX_STR, MAX_LIST, MAX_INTERN>;

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
        let mut interner = DefaultInterner::default();
        let mut lexer = DefaultLexer::default();

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
        let mut interner = DefaultInterner::default();
        let mut lexer = DefaultLexer::default();

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
