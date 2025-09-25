#![no_std]

extern crate alloc;
use alloc::{collections::btree_map::BTreeMap, string::String, vec::Vec};

use core::fmt::Debug;

use misp_common::{
    intern::{InternId, SExprId, StringId},
    sexpr::SExpr,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Capacity exceeded")]
    CapacityExceeded,
    #[error("Invalid ID")]
    InvalidId,
}

pub struct TypeInterner<T, I>
where
    I: InternId,
    T: Ord,
{
    items: Vec<T>,
    map: BTreeMap<T, I>,
}

impl<T, I> Default for TypeInterner<T, I>
where
    I: InternId,
    T: Ord,
{
    fn default() -> Self {
        Self {
            items: Vec::default(),
            map: BTreeMap::default(),
        }
    }
}

impl<T, I> TypeInterner<T, I>
where
    I: InternId,
    T: Ord + Clone,
{
    pub fn intern(&mut self, value: T) -> Result<I, Error> {
        if let Some(&id) = self.map.get(&value) {
            return Ok(id);
        }

        let id = I::from_index(self.items.len());

        self.items.push(value.clone());
        self.map.insert(value, id);

        Ok(id)
    }

    pub fn get(&self, id: I) -> Option<&T> {
        self.items.get(id.to_index())
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn reset(&mut self) {
        self.items.clear();
        self.map.clear();
    }
}

pub type StringInterner = TypeInterner<String, StringId>;
pub type SExprInterner = TypeInterner<SExpr, SExprId>;

#[derive(Default)]
pub struct Interner {
    strings: StringInterner,
    sexprs: SExprInterner,
}

impl Interner {
    pub fn reset(&mut self) {
        // self.strings.reset();
        // self.sexprs.reset();
    }

    pub fn strings(&mut self) -> &mut StringInterner {
        &mut self.strings
    }

    pub fn sexprs(&mut self) -> &mut SExprInterner {
        &mut self.sexprs
    }

    pub fn intern_string(&mut self, s: String) -> Result<StringId, Error> {
        self.strings.intern(s)
    }

    pub fn get_string(&self, id: StringId) -> Option<&str> {
        self.strings.get(id).map(|s| s.as_str())
    }

    pub fn intern_sexpr(&mut self, node: SExpr) -> Result<SExprId, Error> {
        self.sexprs.intern(node)
    }

    pub fn get_sexpr(&self, id: SExprId) -> Option<&SExpr> {
        self.sexprs.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;
    use alloc::{format, string::ToString};
    use misp_common::{
        intern::{InternId, SExprId, StringId},
        sexpr::SExpr,
    };
    use misp_num::decimal::Decimal;

    #[test]
    fn test_type_interner_basic_operations() {
        let mut interner: StringInterner = TypeInterner::default();

        // Test initial state
        assert_eq!(interner.len(), 0);
        assert!(interner.is_empty());

        // Test interning a string
        let hello = "hello".to_string();
        let id1 = interner.intern(hello.clone()).unwrap();

        assert_eq!(interner.len(), 1);
        assert!(!interner.is_empty());
        assert_eq!(interner.get(id1), Some(&hello));
    }

    #[test]
    fn test_string_deduplication() {
        let mut interner: StringInterner = TypeInterner::default();

        let hello1 = "hello".to_string();
        let hello2 = "hello".to_string();
        let world = "world".to_string();

        // Intern same string twice
        let id1 = interner.intern(hello1).unwrap();
        let id2 = interner.intern(hello2).unwrap();
        let id3 = interner.intern(world).unwrap();

        // Same content should get same ID
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);

        // Should only store unique strings
        assert_eq!(interner.len(), 2);

        // Should be able to retrieve both
        assert_eq!(interner.get(id1).unwrap().as_str(), "hello");
        assert_eq!(interner.get(id3).unwrap().as_str(), "world");
    }

    #[test]
    fn test_sexpr_atom_interning() {
        let mut interner = Interner::default();

        // First intern strings
        let test_str_id = interner.intern_string("test".to_string()).unwrap();
        let other_str_id = interner.intern_string("other".to_string()).unwrap();

        // Create atom S-expressions using the string IDs
        let atom1 = SExpr::Atom(test_str_id);
        let atom2 = SExpr::Atom(test_str_id); // Same string ID
        let atom3 = SExpr::Atom(other_str_id); // Different string ID

        let sexpr_id1 = interner.intern_sexpr(atom1).unwrap();
        let sexpr_id2 = interner.intern_sexpr(atom2).unwrap();
        let sexpr_id3 = interner.intern_sexpr(atom3).unwrap();

        // Same S-expressions should get same ID
        assert_eq!(sexpr_id1, sexpr_id2);
        assert_ne!(sexpr_id1, sexpr_id3);

        assert_eq!(interner.sexprs().len(), 2);

        // Verify retrieval
        if let Some(SExpr::Atom(str_id)) = interner.get_sexpr(sexpr_id1) {
            assert_eq!(*str_id, test_str_id);
            assert_eq!(interner.get_string(*str_id), Some("test"));
        } else {
            panic!("Expected atom");
        }
    }

    #[test]
    fn test_sexpr_decimal_interning() {
        let mut interner = Interner::default();

        let decimal1 = SExpr::Decimal(Decimal::from(42));
        let decimal2 = SExpr::Decimal(Decimal::from(42)); // Same value
        let decimal3 = SExpr::Decimal(Decimal::from(24)); // Different value

        let id1 = interner.intern_sexpr(decimal1).unwrap();
        let id2 = interner.intern_sexpr(decimal2).unwrap();
        let id3 = interner.intern_sexpr(decimal3).unwrap();

        // Same decimals should get same ID
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);

        assert_eq!(interner.sexprs().len(), 2);

        // Verify values
        if let Some(SExpr::Decimal(val)) = interner.get_sexpr(id1) {
            assert_eq!(*val, Decimal::from(42));
        } else {
            panic!("Expected decimal");
        }
    }

    #[test]
    fn test_sexpr_list_interning() {
        let mut interner = Interner::default();

        // Create some atoms first
        let plus_str = interner.intern_string("+".to_string()).unwrap();
        let x_str = interner.intern_string("x".to_string()).unwrap();

        let plus_atom = SExpr::Atom(plus_str);
        let x_atom = SExpr::Atom(x_str);
        let num = SExpr::Decimal(Decimal::from(42));

        let plus_id = interner.intern_sexpr(plus_atom).unwrap();
        let x_id = interner.intern_sexpr(x_atom).unwrap();
        let num_id = interner.intern_sexpr(num).unwrap();

        // Create lists: (+ x 42)
        let elements1 = vec![plus_id, x_id, num_id];
        let elements2 = vec![plus_id, x_id, num_id];

        let list1 = SExpr::List(elements1);
        let list2 = SExpr::List(elements2);

        let list_id1 = interner.intern_sexpr(list1).unwrap();
        let list_id2 = interner.intern_sexpr(list2).unwrap();

        // Identical lists should get same ID
        assert_eq!(list_id1, list_id2);

        // Verify structure
        if let Some(SExpr::List(elements)) = interner.get_sexpr(list_id1) {
            assert_eq!(elements.len(), 3);
            assert_eq!(elements[0], plus_id);
            assert_eq!(elements[1], x_id);
            assert_eq!(elements[2], num_id);
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_invalid_id_access() {
        let interner = Interner::default();

        // Test with IDs that don't exist
        let fake_string_id = StringId::from_index(999);
        let fake_sexpr_id = SExprId::from_index(999);

        assert_eq!(interner.get_string(fake_string_id), None);
        assert_eq!(interner.get_sexpr(fake_sexpr_id), None);
    }

    #[test]
    fn test_full_interner_basic_operations() {
        let mut interner = Interner::default();

        // Test string operations
        let string_id = interner.intern_string("hello".to_string()).unwrap();
        assert_eq!(interner.get_string(string_id), Some("hello"));

        // Test S-expression operations using the interned string
        let atom = SExpr::Atom(string_id);
        let sexpr_id = interner.intern_sexpr(atom.clone()).unwrap();
        assert_eq!(interner.get_sexpr(sexpr_id), Some(&atom));
    }

    #[test]
    fn test_string_interner_edge_cases() {
        let mut interner = Interner::default();

        // Test empty string
        let empty_id = interner.intern_string("".to_string()).unwrap();
        assert_eq!(interner.get_string(empty_id), Some(""));

        // Test long string
        let long_string = "a".repeat(1000);
        let long_id = interner.intern_string(long_string.clone()).unwrap();
        assert_eq!(interner.get_string(long_id), Some(long_string.as_str()));
    }

    #[test]
    fn test_multiple_sexpr_types() {
        let mut interner = Interner::default();

        // Create string for atom
        let symbol_str = interner.intern_string("symbol".to_string()).unwrap();

        // Create different types of S-expressions
        let atom = SExpr::Atom(symbol_str);
        let number = SExpr::Decimal(Decimal::from(42));
        let empty_list = SExpr::List(vec![]);

        let atom_id = interner.intern_sexpr(atom.clone()).unwrap();
        let number_id = interner.intern_sexpr(number.clone()).unwrap();
        let list_id = interner.intern_sexpr(empty_list.clone()).unwrap();

        // All should have different IDs
        assert_ne!(atom_id, number_id);
        assert_ne!(atom_id, list_id);
        assert_ne!(number_id, list_id);

        // All should be retrievable
        assert_eq!(interner.get_sexpr(atom_id), Some(&atom));
        assert_eq!(interner.get_sexpr(number_id), Some(&number));
        assert_eq!(interner.get_sexpr(list_id), Some(&empty_list));
    }

    #[test]
    fn test_nested_list_structure() {
        let mut interner = Interner::default();

        // Create: (+ (- x 1) 2)
        let plus_str = interner.intern_string("+".to_string()).unwrap();
        let minus_str = interner.intern_string("-".to_string()).unwrap();
        let x_str = interner.intern_string("x".to_string()).unwrap();

        let plus_atom = interner.intern_sexpr(SExpr::Atom(plus_str)).unwrap();
        let minus_atom = interner.intern_sexpr(SExpr::Atom(minus_str)).unwrap();
        let x_atom = interner.intern_sexpr(SExpr::Atom(x_str)).unwrap();
        let one = interner
            .intern_sexpr(SExpr::Decimal(Decimal::from(1)))
            .unwrap();
        let two = interner
            .intern_sexpr(SExpr::Decimal(Decimal::from(2)))
            .unwrap();

        // Inner list: (- x 1)
        let inner_elements = vec![minus_atom, x_atom, one];
        let inner_list = SExpr::List(inner_elements);
        let inner_list_id = interner.intern_sexpr(inner_list).unwrap();

        // Outer list: (+ (- x 1) 2)
        let outer_elements = vec![plus_atom, inner_list_id, two];
        let outer_list = SExpr::List(outer_elements);
        let outer_list_id = interner.intern_sexpr(outer_list).unwrap();

        // Verify the nested structure
        if let Some(SExpr::List(outer)) = interner.get_sexpr(outer_list_id) {
            assert_eq!(outer.len(), 3);
            assert_eq!(outer[0], plus_atom);
            assert_eq!(outer[1], inner_list_id);
            assert_eq!(outer[2], two);

            // Check inner list
            if let Some(SExpr::List(inner)) = interner.get_sexpr(outer[1]) {
                assert_eq!(inner.len(), 3);
                assert_eq!(inner[0], minus_atom);
                assert_eq!(inner[1], x_atom);
                assert_eq!(inner[2], one);
            } else {
                panic!("Expected inner list");
            }
        } else {
            panic!("Expected outer list");
        }
    }

    #[test]
    fn test_shared_structure_efficiency() {
        let mut interner = Interner::default();

        // Create atoms that will be reused
        let x_str = interner.intern_string("x".to_string()).unwrap();
        let x_atom = interner.intern_sexpr(SExpr::Atom(x_str)).unwrap();

        // Create multiple lists that share the same atom: (x x) and (x x x)
        let list1_elements = vec![x_atom, x_atom];
        let list2_elements = vec![x_atom, x_atom, x_atom];

        let list1 = SExpr::List(list1_elements);
        let list2 = SExpr::List(list2_elements);

        let list1_id = interner.intern_sexpr(list1).unwrap();
        let list2_id = interner.intern_sexpr(list2).unwrap();

        // Should have different list IDs
        assert_ne!(list1_id, list2_id);

        // But should share the same atom reference
        if let (Some(SExpr::List(l1)), Some(SExpr::List(l2))) =
            (interner.get_sexpr(list1_id), interner.get_sexpr(list2_id))
        {
            // All references to x should be the same ID
            assert_eq!(l1[0], x_atom);
            assert_eq!(l1[1], x_atom);
            assert_eq!(l2[0], x_atom);
            assert_eq!(l2[1], x_atom);
            assert_eq!(l2[2], x_atom);
        }

        // Memory efficiency: only one string "x" and one atom SExpr should exist
        assert_eq!(interner.strings().len(), 1); // Just "x"
        assert_eq!(interner.sexprs().len(), 3); // atom(x), list1, list2
    }

    #[test]
    fn test_memory_efficiency() {
        let mut interner = Interner::default();

        // Intern the same string many times
        let test_string = "repeated".to_string();
        let mut string_ids = vec![];

        for _ in 0..10 {
            let id = interner.intern_string(test_string.clone()).unwrap();
            string_ids.push(id);
        }

        // Should all be the same ID
        let first_id = string_ids[0];
        for &id in &string_ids {
            assert_eq!(id, first_id);
        }

        // Should only store one copy
        assert_eq!(interner.strings().len(), 1);

        // Create atoms using the same string ID
        let atom = SExpr::Atom(first_id);
        let mut atom_ids = vec![];

        for _ in 0..10 {
            let id = interner.intern_sexpr(atom.clone()).unwrap();
            atom_ids.push(id);
        }

        // Should all be the same S-expression ID
        let first_atom_id = atom_ids[0];
        for &id in &atom_ids {
            assert_eq!(id, first_atom_id);
        }

        // Should only store one copy of the atom S-expression
        assert_eq!(interner.sexprs().len(), 1);
    }

    #[test]
    fn test_mixed_operations() {
        let mut interner = Interner::default();

        // Interleave string and S-expression operations
        let hello_str_id = interner.intern_string("hello".to_string()).unwrap();
        let world_str_id = interner.intern_string("world".to_string()).unwrap();

        let hello_atom = SExpr::Atom(hello_str_id);
        let world_atom = SExpr::Atom(world_str_id);

        let hello_sexpr_id = interner.intern_sexpr(hello_atom).unwrap();
        _ = interner.intern_sexpr(world_atom).unwrap();

        // More string operations
        let hello_str_id2 = interner.intern_string("hello".to_string()).unwrap();
        assert_eq!(hello_str_id, hello_str_id2);

        // More S-expression operations
        let hello_atom2 = SExpr::Atom(hello_str_id);
        let hello_sexpr_id2 = interner.intern_sexpr(hello_atom2).unwrap();
        assert_eq!(hello_sexpr_id, hello_sexpr_id2);

        // Verify final state
        assert_eq!(interner.strings().len(), 2); // "hello", "world"
        assert_eq!(interner.sexprs().len(), 2); // Atom("hello"), Atom("world")
    }

    #[test]
    fn test_id_consistency() {
        let mut interner = Interner::default();

        // Intern items in order
        let id1 = interner.intern_string("first".to_string()).unwrap();
        let id2 = interner.intern_string("second".to_string()).unwrap();
        let id3 = interner.intern_string("third".to_string()).unwrap();

        // IDs should be sequential
        assert_eq!(id1.to_index(), 0);
        assert_eq!(id2.to_index(), 1);
        assert_eq!(id3.to_index(), 2);

        // Re-interning should give same IDs
        let id1_again = interner.intern_string("first".to_string()).unwrap();
        assert_eq!(id1, id1_again);
    }

    #[test]
    fn test_get_nonexistent_items() {
        let interner = Interner::default();

        // Test with IDs that don't exist
        let fake_string_id = StringId::from_index(999);
        let fake_sexpr_id = SExprId::from_index(999);

        assert_eq!(interner.get_string(fake_string_id), None);
        assert_eq!(interner.get_sexpr(fake_sexpr_id), None);
    }

    #[test]
    fn test_complex_expression() {
        let mut interner = Interner::default();

        // Create a complex expression: (if (> x 0) (+ x 1) (- x 1))

        // Intern symbols
        let if_str = interner.intern_string("if".to_string()).unwrap();
        let gt_str = interner.intern_string(">".to_string()).unwrap();
        let plus_str = interner.intern_string("+".to_string()).unwrap();
        let minus_str = interner.intern_string("-".to_string()).unwrap();
        let x_str = interner.intern_string("x".to_string()).unwrap();

        // Create atoms
        let if_atom = interner.intern_sexpr(SExpr::Atom(if_str)).unwrap();
        let gt_atom = interner.intern_sexpr(SExpr::Atom(gt_str)).unwrap();
        let plus_atom = interner.intern_sexpr(SExpr::Atom(plus_str)).unwrap();
        let minus_atom = interner.intern_sexpr(SExpr::Atom(minus_str)).unwrap();
        let x_atom = interner.intern_sexpr(SExpr::Atom(x_str)).unwrap();
        let zero = interner
            .intern_sexpr(SExpr::Decimal(Decimal::from(0)))
            .unwrap();
        let one = interner
            .intern_sexpr(SExpr::Decimal(Decimal::from(1)))
            .unwrap();

        // Build condition: (> x 0)
        let condition_elements = vec![gt_atom, x_atom, zero];
        let condition = interner
            .intern_sexpr(SExpr::List(condition_elements))
            .unwrap();

        // Build then branch: (+ x 1)
        let then_elements = vec![plus_atom, x_atom, one];
        let then_branch = interner.intern_sexpr(SExpr::List(then_elements)).unwrap();

        // Build else branch: (- x 1)
        let else_elements = vec![minus_atom, x_atom, one];
        let else_branch = interner.intern_sexpr(SExpr::List(else_elements)).unwrap();

        // Build full expression: (if (> x 0) (+ x 1) (- x 1))
        let if_elements = vec![if_atom, condition, then_branch, else_branch];
        let full_expr = interner.intern_sexpr(SExpr::List(if_elements)).unwrap();

        // Verify the structure can be navigated
        if let Some(SExpr::List(if_parts)) = interner.get_sexpr(full_expr) {
            assert_eq!(if_parts.len(), 4);

            // Check condition structure
            if let Some(SExpr::List(cond_parts)) = interner.get_sexpr(if_parts[1]) {
                assert_eq!(cond_parts.len(), 3);
                assert_eq!(cond_parts[0], gt_atom);
                assert_eq!(cond_parts[1], x_atom);
                assert_eq!(cond_parts[2], zero);
            } else {
                panic!("Expected condition list");
            }
        } else {
            panic!("Expected if expression list");
        }

        // Verify memory efficiency - x_atom and one should be shared
        // We should have fewer total S-expressions than if everything was duplicated
        assert!(interner.sexprs().len() < 20); // Much fewer than naive duplication would create
    }

    #[test]
    fn test_large_capacity() {
        let mut interner = Interner::default();

        // Test that we can handle many items without capacity limits
        for i in 0..1000 {
            let string = format!("string_{}", i);
            let string_id = interner.intern_string(string.clone()).unwrap();
            assert_eq!(interner.get_string(string_id), Some(string.as_str()));
        }

        assert_eq!(interner.strings().len(), 1000);
    }
}
