---
applyTo: '**'
---
Architecture Design Document: Merkle-Delta CRDT Store (MDCS)

1. High-Level Design (HLD)

1.1. System Overview

The Merkle-Delta CRDT Store (MDCS) is a novel distributed database architecture engineered for peer-to-peer and collaborative applications. It is specifically designed to deliver high availability, partition tolerance, and robust performance, even when operating over unreliable or high-latency networks. The MDCS establishes a robust foundation for distributed systems by synergizing two core concepts: a Merkle-Clock, which provides a verifiable, tamper-proof causal history, and a δ-CRDT core, which generates the bandwidth-efficient, incremental state changes that are cryptographically secured within that history.

The design of the MDCS is guided by three core architectural goals:

* Deterministic Convergence All replicas are guaranteed to reach an equivalent state once they have delivered the same set of updates. This property, known as Strong Eventual Consistency (SEC), is achieved by modeling the system's state as a monotonic join semilattice. This ensures that merging states is a commutative, associative, and idempotent operation, leading to predictable and deterministic convergence without consensus.
* Open Membership The system is designed to operate in a permissionless, peer-to-peer environment where an arbitrary number of nodes can join and leave. It can tolerate any number of Byzantine (malicious or faulty) nodes. This immunity stems from its ability to tolerate an arbitrary number of Byzantine nodes; since Sybil attacks rely on overwhelming a system with a large number of identities, a design that does not have an upper bound on tolerated faults renders the attack vector moot.
* Bandwidth Efficiency The architecture minimizes network traffic by disseminating small, incremental state changes, known as deltas, instead of transmitting the entire state of an object for each update. Their mathematical properties—idempotence, commutativity, and associativity—allow them to be safely applied in any order, multiple times, and to be joined with the main state or other deltas to achieve convergence.

This architecture is organized into a layered structure that separates concerns from data persistence to application-facing queries, ensuring a modular and maintainable design.

1.2. Logical Architecture

The MDCS is logically structured as a four-layer synthesis. This layered design cleanly separates the system's responsibilities, from the low-level durable storage of data to the high-level reactive interface used by client applications. This separation enhances modularity and simplifies maintenance and future development.

* Storage Layer This is the foundational persistence layer, responsible for the durable and atomic storage of the local replica's state. It manages the physical representation of the Merkle-DAG, which contains the full causal history of all operations, as well as the CRDT data structures that represent the current state of the application's objects.
* Sync Layer This layer orchestrates all inter-replica communication, ensuring that updates are eventually propagated throughout the network. Its two primary components work in tandem: the Broadcaster announces new updates to the network by gossiping the hashes of new DAG heads, which in turn triggers the DAG-Syncer on recipient nodes to discover and fetch any missing parts of the causal history.
* Metadata Layer This is the core of the CRDT implementation, responsible for tracking causality and managing state changes according to convergence rules. Its central component is the Merkle-Clock DAG, a content-addressed graph of updates that embeds the system's complete causal history. This is complemented by a Causal Context, which tracks the set of known updates and enables efficient remove semantics without the use of tombstones.
* Reactivity Layer This is the top-level, application-facing layer. It provides the primary interfaces for applications to query the data store and construct materialized views from the underlying CRDT data structures. This layer allows developers to build responsive user interfaces that reactively update as new state changes are integrated from the network.

This logical architecture defines the internal structure of a single replica and its interaction with the underlying system. The following section details how these replicas interact with each other within a distributed network.

1.3. System Context

MDCS replicas are designed to operate within a distributed environment characterized by an asynchronous, unreliable network. Messages between replicas can be lost, reordered, or experience unbounded delays. The system assumes that while network partitions can occur, they will eventually heal, allowing communication to resume.

The interactions between MDCS replicas can be described as follows:

* Replicas: These are the independent nodes participating in the distributed system. Each replica maintains a full local copy of the MDCS state, including the Merkle-Clock DAG and the CRDT data.
* Network: This is the communication medium connecting the replicas. It is assumed to be unreliable, asynchronous, and subject to partitions.
* Replication Protocol (Asynchronous Gossip): When a replica generates a new update, its Broadcaster component sends the Content Identifier (CID), or hash, of its new DAG head(s) to other peers. This interaction is asynchronous and follows a gossip-based, "push" model to announce new information.
* State Synchronization (Synchronous Fetch): When a replica receives a hash for a DAG node that it does not possess, its DAG-Syncer component initiates a direct request to peers to fetch the missing data block. This is a targeted, synchronous, "pull" interaction designed to repair gaps in the local DAG and ensure causal history is complete.
* Legend: The communication patterns are defined in the table below.

Symbol	Meaning	Communication Type
Solid Lines	Synchronous Calls	Targeted data requests from a DAG-Syncer
Dashed Lines	Asynchronous Gossip	Broadcaster disseminating new update hashes

This high-level system context illustrates the flow of information between replicas, which forms the basis for the detailed internal mechanisms described in the next section.


--------------------------------------------------------------------------------


2. Low-Level Design (LLD)

2.1. The δ-CRDT Core

The δ-CRDT (Delta-CRDT) core is central to the MDCS architecture, enabling both high bandwidth efficiency and the modeling of complex, nested data structures through composition. Instead of propagating full object states, this component generates and disseminates compact, incremental state changes.

The core is defined by two primary mechanisms:

Delta-Mutator Logic An update operation within the MDCS does not modify the local CRDT state directly. Instead, it executes a delta-mutator function that generates a delta. This delta is a small, self-contained data structure representing the specific state change. These deltas are designed to be idempotent, commutative, and associative. This means they can be applied in any order, multiple times, and still produce a convergent state. A delta can be joined with the main state of a replica or with other deltas, ensuring that all replicas eventually reach the same state once all deltas have been delivered.

Recursive Document-Map Structure The MDCS models complex, JSON-like documents using a map that can embed other CRDTs as values. This is achieved by composing a DotMap data structure where keys can be mapped to other CRDTs, such as registers, sets, or even other maps. A critical design choice is the use of a single, shared Causal Context for the entire map and all its nested CRDTs. This shared context is essential for correctly managing causality across the entire document, preventing anomalies that could otherwise arise when keys are removed and subsequently re-added with a new value under concurrent operations.

This delta-based mutation logic is intrinsically linked to the Merkle-Clock, which cryptographically secures these incremental changes within a verifiable causal history.

2.2. Merkle-Clock DAG

The Merkle-Clock is the foundational data structure that establishes a verifiable, partial causal order among all updates in the system. It forms a Directed Acyclic Graph (DAG) where each node represents a discrete update and edges represent causal dependencies.

The Merkle-Clock relies on two key mechanisms:

* Content-Addressed Hashing Every update, which may contain one or more operations, is serialized into a byte string and cryptographically hashed. This hash serves as the update's unique, immutable identifier, or Content Identifier (CID). Each new update references the CIDs of its causal predecessors, forming a DAG structure analogous to a Git commit history. This content-addressed structure provides a powerful guarantee: any two replicas that agree on the set of head CIDs are guaranteed to possess an identical, and therefore consistent, causal history.
* "Gap Repair" Logic Reconciliation between replicas is handled by the DAG-Syncer. When a replica learns of a new DAG head CID from a peer (via the Broadcaster), it traverses the update's list of predecessor hashes. If any predecessor CID is not found in its local store, the DAG-Syncer actively fetches the missing data block from peers. This process continues recursively until the replica's local view of the DAG is complete, thereby repairing any "gaps" in the causal history and allowing the new update to be applied.

The Merkle-Clock ensures that the causal history of the system grows verifiably, while the following mechanisms ensure its size and performance remain manageable over time.

2.3. Compaction & Stability

To ensure the long-term health and performance of a replica, the MDCS incorporates mechanisms to manage the growth of metadata and enable efficient recovery for new or offline nodes.

The following two processes are key to system stability:

* Tombstone-Free Removal The system achieves remove semantics without accumulating tombstones, which can cause unbounded metadata growth in some CRDT set implementations. This is accomplished through a causal context and dot store mechanism. The causal context is a set of all "dots" (unique event IDs) that have been created. The dot store contains only the dots corresponding to currently "live" data. An item is considered removed if its corresponding dot is present in the causal context but absent from the dot store. This design not only prevents unbounded metadata growth but also improves query performance, as lookups do not need to scan through an ever-expanding set of deletion markers.
* Snapshotting and Cold-Starts Bootstrapping a new replica or bringing a long-offline replica up-to-date is managed efficiently through snapshotting. While a replica can reconstruct its state by fetching and replaying the entire Merkle-Clock DAG from genesis, a more practical method is to fetch a recent, compacted snapshot of the state. After applying the snapshot, the replica only needs to fetch and apply the subsequent deltas to become fully synchronized. The system creates these snapshots by deriving a compact version vector from the causal context, which tracks contiguous sequences of updates from each replica. This compressed vector represents the "known delivered frontier" and serves as a safe point for compaction.

These state management mechanisms ensure data integrity and performance, while the Query & View Layer provides application-level access to that data.

2.4. Query & View Layer

The Query & View Layer serves as the primary interface for applications to read and react to data within the MDCS. It translates the internal CRDT structures into formats that are easily consumable by application logic and user interfaces.

This layer provides two main features:

* Prefix Scans The recursive key-value map structure used to model documents naturally supports efficient queries such as prefix scans. This allows applications to retrieve ranges of related data within the document's object tree with a single query, which is useful for fetching collections of items or nested sub-documents.
* Materialized Views The system is designed to support materialized views, which provide a powerful mechanism for building reactive applications. Applications can define specific queries whose results are maintained and updated incrementally as new deltas are applied to the store. This provides a consistently up-to-date view of the data without requiring the application to constantly re-query and re-compute the view from scratch.

This concludes the low-level design overview. The following section provides textual specifications to guide the visualization of these architectural concepts.


--------------------------------------------------------------------------------


3. Visual Specifications

3.1. Diagram Descriptions

This section provides textual descriptions for key architectural diagrams to guide their visualization and ensure a common understanding of the system's structure and data flows.

* System Context Diagram: This diagram illustrates the high-level interaction between replicas. It features several "Replica" nodes positioned around a central "Unreliable Network" cloud. Dashed arrows originate from the replicas, pointing towards the network cloud, to represent asynchronous "DAG Head Gossip." Solid arrows should depict a replica initiating a request through the network to another replica, representing a synchronous "Missing Block Fetch" call.
* Merkle-Clock DAG Structure: This diagram shows a directed acyclic graph representing causal history. Nodes are depicted as circles, each labeled with a unique update hash (e.g., H(A)). Directed edges (arrows) connect nodes to their causal predecessors. The diagram should show an initial state (H(A)) leading to two concurrent updates (H(B) and H(C)), both pointing back to H(A). A subsequent merge update (H(D)) is shown with arrows pointing back to both H(B) and H(C), demonstrating the convergence of concurrent histories.
* Recursive Map and Shared Causal Context: This diagram illustrates the composite data structure for documents. A primary box labeled "Top-Level Map" contains several key-value pairs. One value points to a simple primitive like an LWW-Register. Another value points to a nested box labeled "Nested Map." A separate, distinct box labeled "Shared Causal Context" is shown, with lines connecting it to both the "Top-Level Map" and the "Nested Map," visually reinforcing that the causal context is a single, shared entity for the entire document structure.

3.2. Legend Definition

The following legend provides a definitive key for all symbols used in the architectural diagram descriptions.

Symbol	Element Type	Description
Square	Storage Node	Represents a single, independent replica of the MDCS.
Circle	Event/Vertex	Represents a single update block within the Merkle-Clock DAG.
Arrow	Causal Link	Represents a predecessor hash, linking an update to its causal dependencies.

This visual framework provides a clear model for understanding the system's behavior, which is analyzed in the final section.


--------------------------------------------------------------------------------


4. Technical Requirements Traceability

This final section analyzes how the MDCS architecture explicitly addresses common challenges and limitations found in existing production CRDT implementations, demonstrating its suitability for robust, large-scale deployment.

Addressing Metadata Bloat A significant challenge in CRDT systems that retain history is the unbounded growth of metadata. In contrast to implementations that must store an entire document's history to function correctly, MDCS solves this by actively managing metadata growth. This is achieved through the causal context compression mechanism, which summarizes contiguous update histories from each replica into a compact version vector. This compression, combined with the ability to perform periodic snapshotting based on this vector, allows older, redundant parts of the Merkle-DAG to be garbage collected, ensuring that a replica's storage and memory requirements remain bounded over time.

Preventing Tombstone Accumulation Many CRDT set implementations are plagued by the accumulation of tombstones (deletion markers), which are never removed and lead to performance degradation and increased storage needs. The MDCS architecture solves this problem by design. By using a distinct causal context to track all created events and a separate dot store to track only live events, the system provides efficient remove semantics. An element is considered deleted if its unique identifier exists in the context but not in the store. This design eliminates the need for tombstones entirely, a significant improvement over designs like the original Observed-Remove Set.

Mitigating Network Lag Impact State-based CRDTs, which must transmit the full object state on every update, are fundamentally inefficient on high-latency or lossy networks. The MDCS architecture solves this problem through its δ-CRDT core. By generating and transmitting only small, idempotent deltas that represent the incremental change, the system dramatically reduces network bandwidth requirements. This makes the system highly performant and robust even in challenging network environments, ensuring that applications remain responsive and data continues to sync efficiently.
