#![no_std]
extern crate alloc;

use core::fmt::Display;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use compact_str::CompactString;
use misp_num::decimal::Decimal;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid Token")]
    InvalidToken,
    #[error("Unclosed List")]
    UnclosedList,
    #[error("Unexpected Eof")]
    UnexpectedEof,
}

#[derive(Debug, Clone)]
pub enum SExpr {
    Atom(CompactString),
    List(Vec<SExpr>),
    Decimal(Decimal),
}

#[derive(Debug, Clone)]
pub struct Parser<'a> {
    input: &'a str,
    position: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, position: 0 }
    }

    fn peek(&mut self) -> Option<char> {
        self.input[self.position..].chars().next()
    }

    fn advance(&mut self) {
        if let Some(char) = self.peek() {
            self.position += char.len_utf8();
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_ascii_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_token_slice(&mut self) -> Result<&'a str, Error> {
        let start_pos = self.position;

        while let Some(ch) = self.peek() {
            if ch.is_ascii_whitespace() || ch == '(' || ch == ')' {
                break;
            }
            self.advance();
        }

        if self.position == start_pos {
            return Err(Error::InvalidToken);
        }

        Ok(&self.input[start_pos..self.position])
    }

    fn parse_expr(&mut self) -> Result<SExpr, Error> {
        self.skip_whitespace();

        match self.peek() {
            Some('(') => {
                self.advance();
                let mut sexprs = Vec::new();

                loop {
                    self.skip_whitespace();

                    match self.peek() {
                        None => return Err(Error::UnclosedList),
                        Some(')') => {
                            self.advance();
                            break;
                        }
                        Some(_) => sexprs.push(self.parse_expr()?),
                    }
                }

                Ok(SExpr::List(sexprs))
            }
            Some(_) => {
                let token = self.read_token_slice()?;

                if token
                    .chars()
                    .all(|c| c.is_ascii_digit() || c == '-' || c == '.' || c == '/')
                    && let Ok(decimal) = token.parse::<Decimal>()
                {
                    return Ok(SExpr::Decimal(decimal));
                }

                Ok(SExpr::Atom(token.into()))
            }
            None => Err(Error::UnexpectedEof),
        }
    }

    pub fn parse(&mut self) -> Result<SExpr, Error> {
        self.skip_whitespace();
        self.parse_expr()
    }

    pub fn parse_multiple(&mut self) -> Result<Vec<SExpr>, Error> {
        let mut sexprs = Vec::new();
        loop {
            self.skip_whitespace();

            if self.peek().is_none() {
                break;
            }

            sexprs.push(self.parse_expr()?);
        }

        Ok(sexprs)
    }
}

impl Display for SExpr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SExpr::Atom(s) => write!(f, "{}", s),
            SExpr::List(exprs) => {
                let items: Vec<String> = exprs.iter().map(|e| e.to_string()).collect();
                write!(f, "({})", items.join(" "))
            }
            SExpr::Decimal(d) => write!(f, "{d}",),
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{format, vec};

    use super::*;

    #[test]
    fn test_parse_atom() {
        let mut parser = Parser::new("hello");
        let result = parser.parse().unwrap();

        match result {
            SExpr::Atom(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected atom"),
        }
    }

    #[test]
    fn test_parse_decimal() {
        let mut parser = Parser::new("42");
        let result = parser.parse().unwrap();

        match result {
            SExpr::Decimal(d) => assert_eq!(d, Decimal::from(42)),
            _ => panic!("Expected decimal"),
        }
    }

    #[test]
    fn test_parse_negative_decimal() {
        let mut parser = Parser::new("-42");
        let result = parser.parse().unwrap();

        match result {
            SExpr::Decimal(d) => assert_eq!(d, Decimal::from(-42)),
            _ => panic!("Expected negative decimal"),
        }
    }

    // #[test]
    // fn test_parse_fraction() {
    //     let mut parser = Parser::new("1/2");
    //     let result = parser.parse().unwrap();

    //     match result {
    //         SExpr::Decimal(_) => {} // Just check it parses as decimal
    //         _ => panic!("Expected decimal for fraction"),
    //     }
    // }

    #[test]
    fn test_parse_float() {
        let mut parser = Parser::new("3.14");
        let result = parser.parse().unwrap();

        match result {
            SExpr::Decimal(_) => {}
            _ => panic!("Expected decimal for float"),
        }
    }

    #[test]
    fn test_parse_empty_list() {
        let mut parser = Parser::new("()");
        let result = parser.parse().unwrap();

        match result {
            SExpr::List(items) => assert_eq!(items.len(), 0),
            _ => panic!("Expected empty list"),
        }
    }

    #[test]
    fn test_parse_simple_list() {
        let mut parser = Parser::new("(+ 1 2)");
        let result = parser.parse().unwrap();

        match result {
            SExpr::List(items) => {
                assert_eq!(items.len(), 3);
                assert!(matches!(items[0], SExpr::Atom(ref s) if s == "+"));
                assert!(matches!(items[1], SExpr::Decimal(ref d) if *d == Decimal::from(1)));
                assert!(matches!(items[2], SExpr::Decimal(ref d) if *d == Decimal::from(2)));
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_parse_nested_list() {
        let mut parser = Parser::new("(+ (* 2 3) 4)");
        let result = parser.parse().unwrap();

        match result {
            SExpr::List(items) => {
                assert_eq!(items.len(), 3);
                assert!(matches!(items[0], SExpr::Atom(ref s) if s == "+"));
                assert!(matches!(items[1], SExpr::List(_)));
                assert!(matches!(items[2], SExpr::Decimal(ref d) if *d == Decimal::from(4)));

                // Check nested list
                if let SExpr::List(ref nested) = items[1] {
                    assert_eq!(nested.len(), 3);
                    assert!(matches!(nested[0], SExpr::Atom(ref s) if s == "*"));
                    assert!(matches!(nested[1], SExpr::Decimal(ref d) if *d == Decimal::from(2)));
                    assert!(matches!(nested[2], SExpr::Decimal(ref d) if *d == Decimal::from(3)));
                }
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_parse_multiple_nested_lists() {
        let mut parser = Parser::new("((a b) (c d))");
        let result = parser.parse().unwrap();

        match result {
            SExpr::List(items) => {
                assert_eq!(items.len(), 2);
                assert!(matches!(items[0], SExpr::List(_)));
                assert!(matches!(items[1], SExpr::List(_)));
            }
            _ => panic!("Expected list with nested lists"),
        }
    }

    #[test]
    fn test_parse_with_whitespace() {
        let mut parser = Parser::new("  (  +   1    2  )  ");
        let result = parser.parse().unwrap();

        match result {
            SExpr::List(items) => {
                assert_eq!(items.len(), 3);
                assert!(matches!(items[0], SExpr::Atom(ref s) if s == "+"));
            }
            _ => panic!("Expected list despite whitespace"),
        }
    }

    #[test]
    fn test_parse_mixed_atoms_and_numbers() {
        let mut parser = Parser::new("(define x 42)");
        let result = parser.parse().unwrap();

        match result {
            SExpr::List(items) => {
                assert_eq!(items.len(), 3);
                assert!(matches!(items[0], SExpr::Atom(ref s) if s == "define"));
                assert!(matches!(items[1], SExpr::Atom(ref s) if s == "x"));
                assert!(matches!(items[2], SExpr::Decimal(ref d) if *d == Decimal::from(42)));
            }
            _ => panic!("Expected mixed list"),
        }
    }

    #[test]
    fn test_parse_special_characters_in_atom() {
        let mut parser = Parser::new("hello-world");
        let result = parser.parse().unwrap();

        match result {
            SExpr::Atom(s) => assert_eq!(s, "hello-world"),
            _ => panic!("Expected atom with special characters"),
        }
    }

    #[test]
    fn test_operators_as_atoms() {
        let mut parser = Parser::new("(+ - * /)");
        let result = parser.parse().unwrap();

        match result {
            SExpr::List(items) => {
                assert_eq!(items.len(), 4);
                assert!(matches!(items[0], SExpr::Atom(ref s) if s == "+"));
                assert!(matches!(items[1], SExpr::Atom(ref s) if s == "-"));
                assert!(matches!(items[2], SExpr::Atom(ref s) if s == "*"));
                assert!(matches!(items[3], SExpr::Atom(ref s) if s == "/"));
            }
            _ => panic!("Expected list of operator atoms"),
        }
    }

    // Error cases
    #[test]
    fn test_unclosed_list_error() {
        let mut parser = Parser::new("(+ 1 2");
        let result = parser.parse();

        assert!(matches!(result, Err(Error::UnclosedList)));
    }

    #[test]
    fn test_unexpected_closing_paren() {
        let mut parser = Parser::new(")");
        let result = parser.parse();

        // This should be treated as an invalid token since we start with )
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_input() {
        let mut parser = Parser::new("");
        let result = parser.parse();

        assert!(matches!(result, Err(Error::UnexpectedEof)));
    }

    #[test]
    fn test_whitespace_only() {
        let mut parser = Parser::new("   \t  \n  ");
        let result = parser.parse();

        assert!(matches!(result, Err(Error::UnexpectedEof)));
    }

    #[test]
    fn test_deeply_nested_lists() {
        let mut parser = Parser::new("((((a))))");
        let result = parser.parse().unwrap();

        // Navigate through nested structure
        if let SExpr::List(items1) = result {
            assert_eq!(items1.len(), 1);
            if let SExpr::List(items2) = &items1[0] {
                assert_eq!(items2.len(), 1);
                if let SExpr::List(items3) = &items2[0] {
                    assert_eq!(items3.len(), 1);
                    if let SExpr::List(items4) = &items3[0] {
                        assert_eq!(items4.len(), 1);
                        assert!(matches!(items4[0], SExpr::Atom(ref s) if s == "a"));
                    }
                }
            }
        }
    }

    #[test]
    fn test_display_formatting() {
        // Test atom display
        let atom = SExpr::Atom("hello".into());
        assert_eq!(format!("{}", atom), "hello");

        // Test decimal display
        let decimal = SExpr::Decimal(Decimal::from(42));
        assert_eq!(format!("{}", decimal), "42");

        // Test list display
        let list = SExpr::List(vec![
            SExpr::Atom("+".into()),
            SExpr::Decimal(Decimal::from(1)),
            SExpr::Decimal(Decimal::from(2)),
        ]);
        assert_eq!(format!("{}", list), "(+ 1 2)");

        // Test nested list display
        let nested = SExpr::List(vec![
            SExpr::Atom("+".into()),
            SExpr::List(vec![
                SExpr::Atom("*".into()),
                SExpr::Decimal(Decimal::from(2)),
                SExpr::Decimal(Decimal::from(3)),
            ]),
            SExpr::Decimal(Decimal::from(4)),
        ]);
        assert_eq!(format!("{}", nested), "(+ (* 2 3) 4)");
    }

    #[test]
    fn test_edge_case_single_character_atoms() {
        let mut parser = Parser::new("(a b c)");
        let result = parser.parse().unwrap();

        match result {
            SExpr::List(items) => {
                assert_eq!(items.len(), 3);
                assert!(matches!(items[0], SExpr::Atom(ref s) if s == "a"));
                assert!(matches!(items[1], SExpr::Atom(ref s) if s == "b"));
                assert!(matches!(items[2], SExpr::Atom(ref s) if s == "c"));
            }
            _ => panic!("Expected list of single-char atoms"),
        }
    }

    #[test]
    fn test_mixed_number_formats() {
        let mut parser = Parser::new("(42 -17 3.14)");
        let result = parser.parse().unwrap();

        match result {
            SExpr::List(items) => {
                assert_eq!(items.len(), 3);
                // All should parse as decimals
                assert!(matches!(items[0], SExpr::Decimal(_)));
                assert!(matches!(items[1], SExpr::Decimal(_)));
                assert!(matches!(items[2], SExpr::Decimal(_)));
            }
            _ => panic!("Expected list of decimals"),
        }
    }
}
