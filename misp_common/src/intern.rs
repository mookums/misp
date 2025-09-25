use core::{fmt::Debug, hash::Hash};

pub trait InternId: Debug + Clone + Copy + PartialEq + Eq + Hash {
    fn from_index(index: usize) -> Self;
    fn to_index(self) -> usize;
}

macro_rules! intern_id {
    ($name: ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(usize);

        impl InternId for $name {
            fn from_index(index: usize) -> Self {
                $name(index)
            }

            fn to_index(self) -> usize {
                self.0
            }
        }
    };
}

intern_id!(StringId);
intern_id!(SExprId);
