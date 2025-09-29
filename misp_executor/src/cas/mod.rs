pub mod expand;
pub mod simplify;

#[derive(Debug, Clone, Copy)]
pub enum CasOperation {
    Simplify,
    Expand,
}

pub fn parse_cas_operation(s: &str) -> Option<CasOperation> {
    match s {
        "simplify" => Some(CasOperation::Simplify),
        "expand" => Some(CasOperation::Expand),
        _ => None,
    }
}
