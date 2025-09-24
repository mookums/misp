use misp_num::decimal::Decimal;

use crate::intern::StringId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token<const MAX_STR: usize> {
    LeftParen,
    RightParen,
    Ident(StringId),
    Decimal(Decimal),
}
