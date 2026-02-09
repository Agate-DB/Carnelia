---
applyTo: '**'
---

CRDT and MDCS Architecture Study Guide

Short-Answer Quiz

1. What is a Conflict-free Replicated Data Type (CRDT) and what is its primary purpose in distributed systems?
2. Explain the fundamental difference between "Strongly consistent replication" and "Optimistic replication."
3. What is Strong Eventual Consistency (SEC) and what are the two sufficient conditions for achieving it?
4. The MDCS action plan lists four "non-negotiable principles." What are they and why are they important?
5. What is a δ-CRDT (delta-state CRDT) and how does it differ from a traditional state-based (CvRDT) approach?
6. What is the role of a Merkle-Clock in the proposed MDCS architecture, and what problems does it solve?
7. What is "causal stability" and why is it a critical concept for managing metadata in long-running CRDT systems?
8. The documents identify several recurring "gaps" in real-world CRDT systems. Name and briefly describe three of these gaps.
9. According to the source materials, what is the primary novelty of the proposed MDCS architecture?
10. What is the purpose of using a "differential oracle" or "reference interpreter" in the testing strategy for a CRDT system?

Quiz Answer Key

1. A Conflict-free Replicated Data Type (CRDT) is a data structure designed to simplify distributed data storage systems and multi-user applications. Its primary purpose is to automatically resolve conflicts in systems using optimistic replication, ensuring that data from different replicas can always be merged into a consistent state without special code or user intervention.
2. Strongly consistent replication requires replicas to coordinate with each other before applying modifications, which guarantees consistency models like serializability but reduces performance and availability. In contrast, optimistic replication allows users to modify data on any replica independently, maximizing performance and availability but creating conflicts that must be resolved later.
3. Strong Eventual Consistency (SEC) is a property ensuring that once replicas have received the same set of updates, they converge to an equivalent state regardless of delivery order. The two sufficient conditions are for state-based CRDTs (CvRDTs) to have states that form a join-semilattice, and for operation-based CRDTs (CmRDTs) to deliver operations causally and have concurrent operations commute.
4. The four non-negotiable principles are: 1) Correctness first, performance second; 2) Model the system as hostile (messages can be lost/duplicated/reordered); 3) Every optimization is a semantics change until proven otherwise; and 4) Make compaction/GC an explicit subsystem. They are important for managing the exponential complexity of distributed bugs and ensuring the system's core invariants are maintained.
5. A δ-CRDT is a variant where updates are transmitted as "delta-states" or "delta-mutations" rather than the full state. This is more efficient than a traditional CvRDT which can be expensive when shipping the full state for every synchronization event. Replicas join these deltas into their local state.
6. A Merkle-Clock is a Merkle-DAG used as a logical clock, which decouples causality representation from the number of replicas in the system. In MDCS, its role is to enable open membership (replicas joining/leaving freely), handle unreliable networks by allowing for gap repair, and provide a discovery mechanism for synchronization.
7. Causal stability refers to the property of an operation being provably redundant or having its effects fully reflected in the system's state, such that its associated metadata can be safely removed. This is critical because CRDTs can accumulate large amounts of metadata (tombstones, logs, causal contexts), and causal stability provides a principled way to perform garbage collection (compaction) without affecting correctness.
8. Three recurring gaps are: 1) Transport Assumptions, where op-based CRDTs often need unrealistically strong guarantees like exactly-once reliable causal broadcast; 2) Metadata Growth, where operation logs and tombstones grow unbounded without safe, principled compaction strategies; and 3) Reactivity Degradation, where causal broadcast middleware can buffer operations and delay their application, harming the user experience even when the operations are not semantically dependent.
9. The novelty of MDCS is not in creating new CRDT math, but in its system-level integration of several established research threads. It proposes a coherent database architecture that composes δ-CRDTs for efficient updates, Merkle-Clocks for open-membership sync, a stability-guided compaction strategy for bounded metadata, and a reactivity-aware evaluation path.
10. A differential oracle or reference interpreter is a key part of the test strategy used to verify correctness. It provides a "ground truth" by implementing the CRDT semantics in a simple, slow, but provably correct way (e.g., replaying all operations in a deterministic order). The complex, optimized system is then tested against this oracle to ensure its behavior is identical under various conditions and fault scenarios.

Essay Questions

1. Analyze the proposed MDCS architecture. Discuss how the combination of δ-CRDTs, Merkle-Clocks, and stability-guided compaction aims to address the primary gaps identified in existing CRDT systems and implementations like Yjs and Automerge.
2. The MDCS action plan outlines an eight-phase development process. Describe the logical progression from the foundational CRDT kernel (Phase 1) to a usable database layer (Phase 6), explaining how each phase builds upon the last.
3. Explain the trade-offs between strong consistency (locking) and optimistic replication (lock-free) in the context of collaborative, multi-user applications. Using the provided source material, argue why CRDTs are a suitable foundation for systems in the "local-first" software domain.
4. Discuss the critical role of testing and verification in developing a CRDT-based database. Using the detailed test strategy from the MDCS action plan, explain the importance of property-based invariants, fault model simulation, and differential oracles.
5. The source material repeatedly emphasizes the challenges of metadata growth and garbage collection. Explain what this metadata consists of, why it accumulates, and how the concepts of "causal stability" and "compaction" provide a principled solution to this problem in the MDCS design.

Glossary of Key Terms

Term	Definition
Anti-Entropy	A process in which replicas in a distributed system communicate to compare their states and exchange updates to ensure they eventually converge.
Automerge	A CRDT library for building local-first applications that tracks document history as a series of changes and uses a transport-agnostic sync protocol.
CAP Theorem	A theorem stating that it is impossible for a distributed data store to simultaneously provide more than two out of the following three guarantees: Consistency, Availability, and Partition tolerance. CRDTs prioritize Availability and Partition tolerance.
Causal Consistency	A consistency model that preserves the causal order of operations. If operation A happens before operation B, all replicas that see B must have already seen A.
Causal Stability	A property of an operation indicating that it is safe to remove its associated metadata (e.g., from logs or causal contexts) without affecting future state calculations. It is key to principled garbage collection.
CmRDT (Operation-based CRDT)	A type of CRDT where updates are propagated as operations. It requires a messaging layer that ensures causal delivery and that concurrent operations commute.
Compaction / GC	The process of removing redundant or unnecessary metadata (e.g., operation logs, tombstones, causal context) to manage storage growth. In CRDTs, this must be done carefully to preserve correctness.
Conflict-free Replicated Data Type (CRDT)	A data structure that allows for concurrent modifications on multiple replicas, providing a mathematical guarantee that the replicas can always be merged into a consistent state.
CvRDT (State-based CRDT)	A type of CRDT where the entire state of the data structure is transmitted for synchronization. The state must be a join-semilattice, and the merge operation is the computation of the least upper bound.
Decentralised	An operational model that does not rely on a single central server for communication or coordination. CRDTs are well-suited for decentralised and peer-to-peer networks.
Delta-state CRDT (δ-CRDT)	A CRDT variant that sends incremental "delta-states" or mutations instead of the full state, offering a more efficient way to synchronize replicas. Deltas are idempotent and can be joined into the main state.
Join-Semilattice	A mathematical structure with a set of elements and a join (or merge) operation that is commutative, associative, and idempotent. This structure is a sufficient condition for CvRDT convergence.
MDCS (Merkle‑Delta CRDT Store)	The name of the proposed database architecture that synthesizes δ-CRDTs, Merkle-Clocks, and stability-guided compaction to create an open-membership, offline-first CRDT database.
Merkle-Clock	A logical clock implemented as a Merkle-DAG. It is used to represent the causal history of data in a way that is independent of the number of replicas, making it suitable for open-membership networks.
OpSets	An approach that provides executable semantics for CRDTs by interpreting a set of operations deterministically. It is used as a tool for formal verification and as a reference model for testing.
Optimistic Replication	A replication model where users can modify data on any replica independently, without prior coordination. This model maximizes availability and performance but requires a mechanism, like CRDTs, to resolve conflicts after the fact.
Reactivity	A measure of how quickly a user's actions are reflected in the system's state. In CRDT systems, reactivity can be harmed by causal broadcast middleware that buffers operations, causing perceptible delays.
Replica	A copy of some data stored on a computer in a distributed system.
Strong Eventual Consistency (SEC)	A consistency guarantee provided by CRDTs. It ensures that if all replicas have received the same set of updates, they will converge to an equivalent state.
Strongly Consistent Replication	A replication model where replicas coordinate before applying modifications to ensure strong consistency models like linearizability. This approach sacrifices performance and availability for consistency.
Yjs	A mature CRDT library that uses state vectors and binary updates for an efficient synchronization protocol. Its updates are designed to be commutative, associative, and idempotent.
