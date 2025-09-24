use heapless::Vec;
use misp_num::decimal::Decimal;

use crate::intern::{SExprId, StringId};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum SExpr<const MAX_STR: usize, const MAX_LIST: usize> {
    Atom(StringId),
    List(Vec<SExprId, MAX_LIST>),
    Decimal(Decimal),
}
