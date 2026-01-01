//!  Property-based tests that MUST pass for all CRDT implementations

// use proptest::prelude:: *;
// use mdcs_core::lattice::Lattice;

/// Generate this macro for each CRDT type
macro_rules! lattice_property_tests {
    ($crdt_type:ty, $strategy:expr) => {
        proptest! {
            #[test]
            fn join_is_commutative(a in $strategy. clone(), b in $strategy.clone()) {
                prop_assert_eq!(a.join(&b), b.join(&a));
            }

            #[test]
            fn join_is_associative(
                a in $strategy.clone(),
                b in $strategy.clone(),
                c in $strategy.clone()
            ) {
                let left = a.join(&b).join(&c);
                let right = a.join(&b. join(&c));
                prop_assert_eq!(left, right);
            }

            #[test]
            fn join_is_idempotent(a in $strategy.clone()) {
                prop_assert_eq!(a.join(&a), a);
            }

            #[test]
            fn bottom_is_identity(a in $strategy.clone()) {
                let bottom = <$crdt_type>::bottom();
                prop_assert_eq!(a.join(&bottom), a);
                prop_assert_eq!(bottom.join(&a), a);
            }
        }
    };
}

// Use for each CRDT:
// lattice_property_tests!(GSet<i32>, gset_strategy());
// lattice_property_tests!(ORSet<String>, orset_strategy());