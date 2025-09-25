use core::fmt::Display;

use misp_num::decimal::Decimal;

use crate::intern::{SExprId, StringId};
use alloc::{format, string::String, vec::Vec};

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SExpr {
    Atom(StringId),
    List(Vec<SExprId>),
    Decimal(Decimal),
}

impl Display for SExpr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SExpr::Atom(s) => write!(f, "{s:?}"),
            SExpr::List(exprs) => {
                let items: Vec<String> = exprs.iter().map(|e| format!("{e:?}")).collect();
                write!(f, "({})", items.join(" "))
            }
            SExpr::Decimal(d) => write!(f, "{d}",),
        }
    }
}
