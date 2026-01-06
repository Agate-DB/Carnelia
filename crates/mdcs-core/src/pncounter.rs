// //! PN-Counter (Positive-Negative Counter) CRDT
// //!
// //! A PN-Counter supports both increment and decrement operations by maintaining
// //! two G-Counters: one for increments (P) and one for decrements (N).
// //! The value is P - N.
//
// use std::collections::HashMap;
// use std::hash::Hash;
// use serde::{Deserialize, Serialize};
//
// use crate::gset::GSet;
// use crate::lattice::Lattice;
//
// /// A Positive-Negative Counter CRDT
// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// pub struct PNCounter<K: Eq + Hash + Clone> {
//     /// Positive counter (increments)
//     p: GSet<K>,
//     /// Negative counter (decrements)
//     n: GSet<K>,
// }
//
// impl<K: Eq + Hash + Clone> PNCounter<K> {
//     /// Create a new PN-Counter for the given node
//     pub fn new(node_id: K) -> Self {
//         Self {
//             p: GSet::new(node_id.clone()),
//             n: GSet::new(node_id),
//         }
//     }
//
//     /// Increment the counter by 1
//     pub fn increment(&mut self) {
//         self.p.increment();
//     }
//
//     /// Increment the counter by a specific amount
//     pub fn increment_by(&mut self, amount: u64) {
//         self.p.increment_by(amount);
//     }
//
//     /// Decrement the counter by 1
//     pub fn decrement(&mut self) {
//         self.n.increment();
//     }
//
//     /// Decrement the counter by a specific amount
//     pub fn decrement_by(&mut self, amount: u64) {
//         self.n.increment_by(amount);
//     }
//
//     /// Get the current value of the counter (can be negative)
//     pub fn value(&self) -> i64 {
//         self.p.value() as i64 - self.n.value() as i64
//     }
//
//     /// Merge another PN-Counter into this one
//     pub fn merge(&mut self, other: &PNCounter<K>) {
//         self.p.merge(&other.p);
//         self.n.merge(&other.n);
//     }
//
//     /// Compare two PN-Counters (returns true if self <= other)
//     pub fn le(&self, other: &PNCounter<K>) -> bool {
//         self.p.le(&other.p) && self.n.le(&other.n)
//     }
// }
//
// impl<K: Eq + Hash + Clone> Lattice for PNCounter<K> {
//     fn bottom() -> Self {
//         Self {
//             p: GSet::bottom(),
//             n: GSet::bottom(),
//         }
//     }
//
//     fn join(&self, other: &Self) -> Self {
//         Self {
//             p: self.p.join(&other.p),
//             n: self.n.join(&other.n),
//         }
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_increment_decrement() {
//         let mut counter = PNCounter::new("node1");
//         assert_eq!(counter.value(), 0);
//
//         counter.increment_by(10);
//         assert_eq!(counter.value(), 10);
//
//         counter.decrement_by(3);
//         assert_eq!(counter.value(), 7);
//     }
//
//     #[test]
//     fn test_negative_value() {
//         let mut counter = PNCounter::new("node1");
//         counter.decrement_by(5);
//         assert_eq!(counter.value(), -5);
//     }
//
//     #[test]
//     fn test_merge() {
//         let mut counter1 = PNCounter::new("node1");
//         let mut counter2 = PNCounter::new("node2");
//
//         counter1.increment_by(10);
//         counter1.decrement_by(2);
//
//         counter2.increment_by(5);
//         counter2.decrement_by(1);
//
//         counter1.merge(&counter2);
//         assert_eq!(counter1.value(), 12); // (10 + 5) - (2 + 1) = 12
//     }
//
//     #[test]
//     fn test_lattice_join() {
//         let mut counter1 = PNCounter::new("node1");
//         let mut counter2 = PNCounter::new("node2");
//
//         counter1.increment_by(5);
//         counter2.decrement_by(3);
//
//         let joined = counter1.join(&counter2);
//         assert_eq!(joined.value(), 2); // 5 - 3 = 2
//     }
// }