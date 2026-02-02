//! Property-based tests that MUST pass for all CRDT implementations
//!
//! These tests verify the lattice laws that guarantee convergence:
//!  - Commutativity: a ⊔ b = b ⊔ a
//!  - Associativity: (a ⊔ b) ⊔ c = a ⊔ (b ⊔ c)
//!  - Idempotence:  a ⊔ a = a
//!  - Bottom is identity: a ⊔ ⊥ = a

use proptest::prelude::*;
use mdcs_core::lattice::{Lattice, DeltaCRDT};
use mdcs_core::gset::GSet;
use mdcs_core::orset::ORSet;
use mdcs_core::pncounter::PNCounter;
use mdcs_core::lwwreg::LWWRegister;
use mdcs_core::mvreg::MVRegister;

/// Generate strategies for prop-testing

fn gset_i32_strategy() -> impl Strategy<Value = GSet<i32>> {
    prop::collection::btree_set(0i32..100, 0..20)
        .prop_map(|elements| {
            let mut set = GSet::new();
            for e in elements {
                set.insert(e);
            }
            set
        })
}

fn orset_string_strategy() -> impl Strategy<Value = ORSet<String>> {
    prop::collection::vec("[a-z]{1,5}", 0..10)
        .prop_map(|elements| {
            let mut set = ORSet::new();
            for (i, e) in elements.iter().enumerate() {
                set.add(&format!("replica{}", i % 3), e.clone());
            }
            // Clear pending delta so equality comparisons work correctly
            let _ = set.split_delta();
            set
        })
}

fn pncounter_strategy() -> impl Strategy<Value = PNCounter<String>> {
    (0u64..100, 0u64..50)
        .prop_map(|(inc, dec)| {
            let mut counter = PNCounter::new();
            counter.increment("replica1".to_string(), inc);
            counter.decrement("replica2".to_string(), dec);
            counter
        })
}

fn lwwreg_strategy() -> impl Strategy<Value = LWWRegister<i32, String>> {
    (0i32..100, 0u64..1000)
        .prop_map(|(value, timestamp)| {
            let mut reg = LWWRegister::new("replica1".to_string());
            reg.set(value, timestamp, "replica1".to_string());
            reg
        })
}

fn mvreg_strategy() -> impl Strategy<Value = MVRegister<i32>> {
    (0i32..100)
        .prop_map(|value| {
            let mut reg = MVRegister::new();
            reg.write("replica1", value);
            reg
        })
}

// ============================================================================
// GSet Property Tests
// ============================================================================

proptest! {
    #[test]
    fn gset_join_is_commutative(
        a in gset_i32_strategy(),
        b in gset_i32_strategy()
    ) {
        prop_assert_eq!(a.join(&b), b.join(&a));
    }

    #[test]
    fn gset_join_is_associative(
        a in gset_i32_strategy(),
        b in gset_i32_strategy(),
        c in gset_i32_strategy()
    ) {
        let left = a.join(&b).join(&c);
        let right = a.join(&b.join(&c));
        prop_assert_eq!(left, right);
    }

    #[test]
    fn gset_join_is_idempotent(a in gset_i32_strategy()) {
        prop_assert_eq!(a.join(&a), a);
    }

    #[test]
    fn gset_bottom_is_identity(a in gset_i32_strategy()) {
        let bottom = GSet::bottom();
        prop_assert_eq!(a.join(&bottom), a.clone());
        prop_assert_eq!(bottom.join(&a), a);
    }
}

// ============================================================================
// ORSet Property Tests
// ============================================================================

proptest! {
    #[test]
    fn orset_join_is_commutative(
        a in orset_string_strategy(),
        b in orset_string_strategy()
    ) {
        prop_assert_eq!(a.join(&b), b.join(&a));
    }

    #[test]
    fn orset_join_is_associative(
        a in orset_string_strategy(),
        b in orset_string_strategy(),
        c in orset_string_strategy()
    ) {
        let left = a.join(&b).join(&c);
        let right = a.join(&b.join(&c));
        prop_assert_eq!(left, right);
    }

    #[test]
    fn orset_join_is_idempotent(a in orset_string_strategy()) {
        prop_assert_eq!(a.join(&a), a);
    }

    #[test]
    fn orset_bottom_is_identity(a in orset_string_strategy()) {
        let bottom = ORSet::bottom();
        prop_assert_eq!(a.join(&bottom), a.clone());
        prop_assert_eq!(bottom.join(&a), a);
    }
}

// ============================================================================
// PNCounter Property Tests
// ============================================================================

proptest! {
    #[test]
    fn pncounter_join_is_commutative(
        a in pncounter_strategy(),
        b in pncounter_strategy()
    ) {
        prop_assert_eq!(a.join(&b), b.join(&a));
    }

    #[test]
    fn pncounter_join_is_associative(
        a in pncounter_strategy(),
        b in pncounter_strategy(),
        c in pncounter_strategy()
    ) {
        let left = a.join(&b).join(&c);
        let right = a.join(&b.join(&c));
        prop_assert_eq!(left, right);
    }

    #[test]
    fn pncounter_join_is_idempotent(a in pncounter_strategy()) {
        prop_assert_eq!(a.join(&a), a);
    }

    #[test]
    fn pncounter_bottom_is_identity(a in pncounter_strategy()) {
        let bottom = PNCounter::bottom();
        prop_assert_eq!(a.join(&bottom), a.clone());
        prop_assert_eq!(bottom.join(&a), a);
    }

    #[test]
    fn pncounter_value_convergence(
        a in pncounter_strategy(),
        b in pncounter_strategy()
    ) {
        let joined1 = a.join(&b);
        let joined2 = b.join(&a);
        // Values must converge regardless of order
        prop_assert_eq!(joined1.value(), joined2.value());
    }
}

// ============================================================================
// LWWRegister Property Tests
// ============================================================================

proptest! {
    #[test]
    fn lwwreg_join_is_commutative(
        a in lwwreg_strategy(),
        b in lwwreg_strategy()
    ) {
        prop_assert_eq!(a.join(&b), b.join(&a));
    }

    #[test]
    fn lwwreg_join_is_associative(
        a in lwwreg_strategy(),
        b in lwwreg_strategy(),
        c in lwwreg_strategy()
    ) {
        let left = a.join(&b).join(&c);
        let right = a.join(&b.join(&c));
        prop_assert_eq!(left, right);
    }

    #[test]
    fn lwwreg_join_is_idempotent(a in lwwreg_strategy()) {
        prop_assert_eq!(a.join(&a), a);
    }

    #[test]
    fn lwwreg_bottom_is_identity(a in lwwreg_strategy()) {
        let bottom = LWWRegister::bottom();
        prop_assert_eq!(a.join(&bottom), a.clone());
        prop_assert_eq!(bottom.join(&a), a);
    }
}

// ============================================================================
// MVRegister Property Tests
// ============================================================================

proptest! {
    #[test]
    fn mvreg_join_is_commutative(
        a in mvreg_strategy(),
        b in mvreg_strategy()
    ) {
        prop_assert_eq!(a.join(&b), b.join(&a));
    }

    #[test]
    fn mvreg_join_is_associative(
        a in mvreg_strategy(),
        b in mvreg_strategy(),
        c in mvreg_strategy()
    ) {
        let left = a.join(&b).join(&c);
        let right = a.join(&b.join(&c));
        prop_assert_eq!(left, right);
    }

    #[test]
    fn mvreg_join_is_idempotent(a in mvreg_strategy()) {
        prop_assert_eq!(a.join(&a), a);
    }

    #[test]
    fn mvreg_bottom_is_identity(a in mvreg_strategy()) {
        let bottom = MVRegister::bottom();
        prop_assert_eq!(a.join(&bottom), a.clone());
        prop_assert_eq!(bottom.join(&a), a);
    }
}

// ============================================================================
// Serialization Round-Trip Tests
// ============================================================================

#[test]
fn gset_serialization_roundtrip() {
    let mut set = GSet::new();
    set.insert(1);
    set.insert(2);
    set.insert(3);

    let serialized = serde_json::to_string(&set).unwrap();
    let deserialized: GSet<i32> = serde_json::from_str(&serialized).unwrap();

    assert_eq!(set, deserialized);
}

#[test]
fn orset_serialization_roundtrip() {
    let mut set = ORSet::new();
    set.add("replica1", "hello".to_string());
    set.add("replica2", "world".to_string());
    // Clear pending delta before serialization comparison
    let _ = set.split_delta();

    let serialized = serde_json::to_string(&set).unwrap();
    let deserialized: ORSet<String> = serde_json::from_str(&serialized).unwrap();

    assert_eq!(set, deserialized);
}

#[test]
fn pncounter_serialization_roundtrip() {
    let mut counter = PNCounter::new();
    counter.increment("replica1".to_string(), 42);
    counter.decrement("replica2".to_string(), 10);

    let serialized = serde_json::to_string(&counter).unwrap();
    let deserialized: PNCounter<String> = serde_json::from_str(&serialized).unwrap();

    assert_eq!(counter, deserialized);
    assert_eq!(counter.value(), deserialized.value());
}

#[test]
fn lwwreg_serialization_roundtrip() {
    let mut reg = LWWRegister::new("replica1".to_string());
    reg.set(42, 100, "replica1".to_string());

    let serialized = serde_json::to_string(&reg).unwrap();
    let deserialized: LWWRegister<i32, String> = serde_json::from_str(&serialized).unwrap();

    assert_eq!(reg, deserialized);
}

#[test]
fn mvreg_serialization_roundtrip() {
    let mut reg = MVRegister::new();
    reg.write("replica1", 42);

    let serialized = serde_json::to_string(&reg).unwrap();
    let deserialized: MVRegister<i32> = serde_json::from_str(&serialized).unwrap();

    assert_eq!(reg, deserialized);
}