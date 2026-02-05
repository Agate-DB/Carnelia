//! # Carnelia Increment
//!
//! A standalone CLI increment tracker built on the MDCS SDK.
//! Uses `JsonDoc` to implement distributed counters: each replica stores its
//! own contribution under a namespaced path, making concurrent increments
//! conflict-free and perfectly mergeable.
//!
//! ## Counter model (PN-Counter over JSON)
//!
//! ```text
//! path: /counters/<name>/<replica_id>/inc   →  JsonValue::Int(n)
//! path: /counters/<name>/<replica_id>/dec   →  JsonValue::Int(n)
//! total = Σ(inc across all replicas) − Σ(dec across all replicas)
//! ```

use std::collections::HashMap;
use std::io::{self, Write};

use clap::{Parser, Subcommand};
use colored::*;
use mdcs_sdk::document::JsonDoc;
use mdcs_sdk::JsonValue;

// ─── CLI ───────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(name = "carnelia-increment")]
#[command(about = "CRDT-based distributed increment tracker (MDCS SDK)")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Basic demo: two replicas increment, sync, and converge
    Demo,
    /// Conflict scenario: concurrent updates across 3 replicas, merge-order independence
    Conflict,
    /// Network partition simulation: split, independent work, heal, full convergence
    Partition,
    /// Interactive REPL for manual experimentation
    Interactive,
}

// ─── Replica: a simulated node holding a JsonDoc ───────────────────────────

/// Each Replica owns a `JsonDoc` and stores counters under namespaced paths.
/// The path layout is:
///   `/counters/{counter_name}/{replica_id}/inc`  → cumulative increments
///   `/counters/{counter_name}/{replica_id}/dec`  → cumulative decrements
struct Replica {
    id: String,
    doc: JsonDoc,
}

impl Replica {
    fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            doc: JsonDoc::new("counters", id),
        }
    }

    /// Current increment contribution for a counter from this replica.
    fn own_inc(&self, counter: &str) -> i64 {
        let path = format!("counters.{}.{}.inc", counter, self.id);
        match self.doc.get(&path) {
            Some(JsonValue::Int(n)) => n,
            _ => 0,
        }
    }

    /// Current decrement contribution for a counter from this replica.
    fn own_dec(&self, counter: &str) -> i64 {
        let path = format!("counters.{}.{}.dec", counter, self.id);
        match self.doc.get(&path) {
            Some(JsonValue::Int(n)) => n,
            _ => 0,
        }
    }

    /// Increment a named counter by `amount`.
    fn increment(&mut self, counter: &str, amount: i64) {
        let new_val = self.own_inc(counter) + amount;
        let path = format!("counters.{}.{}.inc", counter, self.id);
        self.doc.set(&path, JsonValue::Int(new_val));
    }

    /// Decrement a named counter by `amount`.
    fn decrement(&mut self, counter: &str, amount: i64) {
        let new_val = self.own_dec(counter) + amount;
        let path = format!("counters.{}.{}.dec", counter, self.id);
        self.doc.set(&path, JsonValue::Int(new_val));
    }

    /// Compute total value for a counter across all replicas visible in this doc.
    fn value(&self, counter: &str) -> i64 {
        // Walk the JSON tree: root.counters.<counter>.<replica>.{inc,dec}
        let root = self.doc.root();
        let counter_obj = match root.get("counters").and_then(|c| c.get(counter)) {
            Some(obj) => obj,
            None => return 0,
        };

        let mut total: i64 = 0;
        if let Some(map) = counter_obj.as_object() {
            for (_replica_id, replica_data) in map {
                if let Some(inc) = replica_data.get("inc").and_then(|v| v.as_i64()) {
                    total += inc;
                }
                if let Some(dec) = replica_data.get("dec").and_then(|v| v.as_i64()) {
                    total -= dec;
                }
            }
        }
        total
    }

    /// Discover all counter names present in the document.
    fn counter_names(&self) -> Vec<String> {
        let root = self.doc.root();
        let mut names: Vec<String> = match root.get("counters").and_then(|c| c.as_object()) {
            Some(map) => map.keys().cloned().collect(),
            None => Vec::new(),
        };
        names.sort();
        names
    }

    /// Collect per-replica breakdown for a counter.
    fn breakdown(&self, counter: &str) -> Vec<(String, i64, i64)> {
        let root = self.doc.root();
        let counter_obj = match root.get("counters").and_then(|c| c.get(counter)) {
            Some(obj) => obj,
            None => return Vec::new(),
        };

        let mut result: Vec<(String, i64, i64)> = Vec::new();
        if let Some(map) = counter_obj.as_object() {
            for (replica_id, replica_data) in map {
                let inc = replica_data.get("inc").and_then(|v| v.as_i64()).unwrap_or(0);
                let dec = replica_data.get("dec").and_then(|v| v.as_i64()).unwrap_or(0);
                result.push((replica_id.clone(), inc, dec));
            }
        }
        result.sort_by(|a, b| a.0.cmp(&b.0));
        result
    }

    /// CRDT merge: apply another replica's state into this one.
    fn sync_from(&mut self, other: &Replica) {
        let snapshot = other.doc.clone_state();
        self.doc.merge(&snapshot);
    }
}

// ─── Pretty printing ──────────────────────────────────────────────────────

fn header(text: &str) {
    let bar = "═".repeat(60);
    println!("\n{}", bar.bright_cyan());
    println!("  {}", text.bold().bright_white());
    println!("{}", bar.bright_cyan());
}

fn section(text: &str) {
    println!("\n{} {}", "▸".bright_yellow(), text.bold());
}

fn step(text: &str) {
    println!("  {} {}", "•".bright_green(), text);
}

fn sync_arrow(from: &str, to: &str) {
    println!(
        "  {} {} {} {}",
        from.bright_magenta(),
        "──sync──▶".bright_cyan(),
        to.bright_magenta(),
        "✓".bright_green()
    );
}

fn show_replica(replica: &Replica) {
    let border = "─".repeat(44);
    println!("  ┌{}┐", border);
    println!(
        "  │ {:^42} │",
        format!("Replica: {}", replica.id).bright_yellow().to_string()
    );
    println!("  ├{}┤", border);

    let names = replica.counter_names();
    if names.is_empty() {
        println!("  │ {:^42} │", "(no counters)".dimmed().to_string());
    } else {
        for name in &names {
            let val = replica.value(name);
            let breakdown = replica.breakdown(name);
            let parts: Vec<String> = breakdown
                .iter()
                .map(|(rid, inc, dec)| {
                    if *dec > 0 {
                        format!("{}:+{}-{}", rid, inc, dec)
                    } else {
                        format!("{}:+{}", rid, inc)
                    }
                })
                .collect();
            let detail = parts.join(", ");
            let line = format!("  {:<16} = {:>5}  ({})", name, val, detail);
            let padded = format!("{:<42}", line);
            println!("  │ {} │", padded);
        }
    }
    println!("  └{}┘", border);
}

fn convergence_check(replicas: &[&Replica]) -> bool {
    if replicas.len() < 2 {
        return true;
    }
    let base = &replicas[0];
    let names = base.counter_names();
    for r in &replicas[1..] {
        for name in &names {
            if base.value(name) != r.value(name) {
                return false;
            }
        }
        // Also check the other replica doesn't have extra counters
        for name in &r.counter_names() {
            if base.value(name) != r.value(name) {
                return false;
            }
        }
    }
    true
}

fn convergence_result(converged: bool) {
    if converged {
        println!(
            "\n  {} {}",
            "✓".bright_green().bold(),
            "ALL REPLICAS CONVERGED — values are identical!"
                .bright_green()
                .bold()
        );
    } else {
        println!(
            "\n  {} {}",
            "✗".bright_red().bold(),
            "DIVERGENCE DETECTED — replicas differ!"
                .bright_red()
                .bold()
        );
    }
}

// ─── Demo ──────────────────────────────────────────────────────────────────

fn run_demo() {
    header("DEMO — Basic Increment Tracking & CRDT Sync");

    section("Phase 1: Two replicas increment independently");
    let mut alice = Replica::new("alice");
    let mut bob = Replica::new("bob");

    alice.increment("page_views", 5);
    step("alice: page_views += 5");
    alice.increment("page_views", 3);
    step("alice: page_views += 3  (total contribution: 8)");

    bob.increment("page_views", 10);
    step("bob:   page_views += 10");
    bob.increment("likes", 2);
    step("bob:   likes += 2");

    show_replica(&alice);
    show_replica(&bob);

    section("Phase 2: Bidirectional sync via CRDT merge");
    alice.sync_from(&bob);
    sync_arrow("bob", "alice");
    bob.sync_from(&alice);
    sync_arrow("alice", "bob");

    section("Phase 3: Post-sync state");
    show_replica(&alice);
    show_replica(&bob);

    let ok = convergence_check(&[&alice, &bob]);
    convergence_result(ok);

    section("Final values");
    step(&format!(
        "page_views = {} (alice:8 + bob:10)",
        alice.value("page_views")
    ));
    step(&format!("likes = {} (bob:2)", alice.value("likes")));
}

// ─── Conflict ──────────────────────────────────────────────────────────────

fn run_conflict() {
    header("CONFLICT — Concurrent Updates, Merge-Order Independence");

    section("Phase 1: Three replicas make concurrent edits to the same counters");
    let mut r1 = Replica::new("node-1");
    let mut r2 = Replica::new("node-2");
    let mut r3 = Replica::new("node-3");

    r1.increment("score", 100);
    r1.decrement("score", 10);
    step("node-1: score += 100, score -= 10");

    r2.increment("score", 50);
    r2.increment("bonus", 25);
    step("node-2: score += 50, bonus += 25");

    r3.increment("score", 75);
    r3.decrement("score", 5);
    r3.increment("bonus", 10);
    step("node-3: score += 75, score -= 5, bonus += 10");

    section("Pre-sync (diverged)");
    show_replica(&r1);
    show_replica(&r2);
    show_replica(&r3);

    section("Phase 2: Merge in 3 different orders to prove commutativity");

    // Order A: r1 ← r2 ← r3
    let mut order_a = Replica::new("order-A");
    order_a.sync_from(&r1);
    order_a.sync_from(&r2);
    order_a.sync_from(&r3);
    step(&format!(
        "Order A (r1→r2→r3): score={}, bonus={}",
        order_a.value("score"),
        order_a.value("bonus")
    ));

    // Order B: r3 ← r1 ← r2
    let mut order_b = Replica::new("order-B");
    order_b.sync_from(&r3);
    order_b.sync_from(&r1);
    order_b.sync_from(&r2);
    step(&format!(
        "Order B (r3→r1→r2): score={}, bonus={}",
        order_b.value("score"),
        order_b.value("bonus")
    ));

    // Order C: r2 ← r3 ← r1
    let mut order_c = Replica::new("order-C");
    order_c.sync_from(&r2);
    order_c.sync_from(&r3);
    order_c.sync_from(&r1);
    step(&format!(
        "Order C (r2→r3→r1): score={}, bonus={}",
        order_c.value("score"),
        order_c.value("bonus")
    ));

    let ok = convergence_check(&[&order_a, &order_b, &order_c]);
    section("Merge-order independence");
    convergence_result(ok);
    step(&format!(
        "score = {} (100+50+75 − 10−5 = 210)",
        order_a.value("score")
    ));
    step(&format!(
        "bonus = {} (25+10 = 35)",
        order_a.value("bonus")
    ));

    section("Phase 3: Idempotence — merging the same state twice");
    let before = order_a.value("score");
    order_a.sync_from(&r1);
    order_a.sync_from(&r1); // intentional duplicate
    let after = order_a.value("score");
    if before == after {
        step(&format!(
            "Idempotent ✓  score stayed {} after duplicate merges",
            after
        ));
    } else {
        step(&format!("IDEMPOTENCE FAILURE: {} → {} ✗", before, after));
    }
}

// ─── Partition ─────────────────────────────────────────────────────────────

fn run_partition() {
    header("PARTITION — Network Split, Independent Work, Heal & Converge");

    section("Phase 1: Create 4 replicas in 2 data-centers, establish shared baseline");
    let mut east1 = Replica::new("east-1");
    let mut east2 = Replica::new("east-2");
    let mut west1 = Replica::new("west-1");
    let mut west2 = Replica::new("west-2");

    east1.increment("requests", 100);
    // Sync initial state to all
    east2.sync_from(&east1);
    west1.sync_from(&east1);
    west2.sync_from(&east1);
    step("Baseline: requests = 100, synced to all 4 replicas");

    section("Phase 2: NETWORK PARTITION");
    println!(
        "  {}   {}",
        "╔══════════════════╗".bright_blue(),
        "╔══════════════════╗".bright_red()
    );
    println!(
        "  {}   {}",
        "║  EAST DC         ║".bright_blue(),
        "║  WEST DC         ║".bright_red()
    );
    println!(
        "  {}   {}",
        "║  east-1, east-2  ║".bright_blue(),
        "║  west-1, west-2  ║".bright_red()
    );
    println!(
        "  {}   {}",
        "╚══════════════════╝".bright_blue(),
        "╚══════════════════╝".bright_red()
    );
    println!(
        "  {}",
        "         ╳╳╳ PARTITION ╳╳╳".bright_red().bold()
    );

    // East-side work
    east1.increment("requests", 50);
    east1.increment("errors", 3);
    east2.increment("requests", 30);
    east2.decrement("errors", 1);
    east1.sync_from(&east2);
    east2.sync_from(&east1);
    step("East: east-1 +50 req, +3 err; east-2 +30 req, −1 err correction");
    step("East internal sync complete");

    // West-side work
    west1.increment("requests", 200);
    west1.increment("latency_spikes", 7);
    west2.increment("requests", 150);
    west2.increment("latency_spikes", 3);
    west1.sync_from(&west2);
    west2.sync_from(&west1);
    step("West: west-1 +200 req, +7 spikes; west-2 +150 req, +3 spikes");
    step("West internal sync complete");

    section("Pre-heal state");
    show_replica(&east1);
    show_replica(&west1);

    section("Phase 3: PARTITION HEALS");
    println!(
        "  {}",
        "         ════ HEALED ════".bright_green().bold()
    );

    // Take snapshots to avoid borrow issues, then full mesh sync
    let east_snap = Replica {
        id: "east-snap".to_string(),
        doc: east1.doc.clone_state(),
    };
    let west_snap = Replica {
        id: "west-snap".to_string(),
        doc: west1.doc.clone_state(),
    };

    for r in [&mut east1, &mut east2, &mut west1, &mut west2] {
        r.sync_from(&east_snap);
        r.sync_from(&west_snap);
    }
    sync_arrow("east", "west");
    sync_arrow("west", "east");
    step("Full mesh sync across all 4 replicas");

    section("Phase 4: Post-heal state");
    show_replica(&east1);
    show_replica(&west1);

    let ok = convergence_check(&[&east1, &east2, &west1, &west2]);
    convergence_result(ok);

    step(&format!(
        "requests       = {} (100+50+30+200+150 = 530)",
        east1.value("requests")
    ));
    step(&format!(
        "errors         = {} (3−1 = 2)",
        east1.value("errors")
    ));
    step(&format!(
        "latency_spikes = {} (7+3 = 10)",
        east1.value("latency_spikes")
    ));
}

// ─── Interactive REPL ──────────────────────────────────────────────────────

fn run_interactive() {
    header("INTERACTIVE REPL — MDCS SDK Increment Tracker");

    let mut replicas: HashMap<String, Replica> = HashMap::new();

    println!();
    println!("  {}", "Commands:".bold().underline());
    println!(
        "    {} <name>                   Create a new replica",
        "replica".bright_cyan()
    );
    println!(
        "    {} <replica> <counter> [n]   Increment counter by n (default 1)",
        "inc".bright_cyan()
    );
    println!(
        "    {} <replica> <counter> [n]   Decrement counter by n (default 1)",
        "dec".bright_cyan()
    );
    println!(
        "    {} <from> <to>              Merge from → to",
        "sync".bright_cyan()
    );
    println!(
        "    {} <name>                 Sync bidirectionally with all others",
        "syncall".bright_cyan()
    );
    println!(
        "    {} <name>                  Show replica state",
        "show".bright_cyan()
    );
    println!(
        "    {}                          Show all replicas",
        "list".bright_cyan()
    );
    println!(
        "    {} <r1> <r2>               Check convergence between two replicas",
        "check".bright_cyan()
    );
    println!(
        "    {} <name> <counter>        Show per-replica breakdown",
        "detail".bright_cyan()
    );
    println!(
        "    {}                          Exit",
        "quit".bright_cyan()
    );
    println!();

    loop {
        print!("{}", "carnelia> ".bright_cyan().bold());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() || input.is_empty() {
            break;
        }
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "replica" | "r" => {
                if parts.len() < 2 {
                    println!("  {} Usage: replica <name>", "!".bright_red());
                    continue;
                }
                let name = parts[1];
                if replicas.contains_key(name) {
                    println!("  {} Replica '{}' already exists", "!".bright_yellow(), name);
                } else {
                    replicas.insert(name.to_string(), Replica::new(name));
                    step(&format!("Created replica '{}'", name));
                }
            }

            "inc" | "+" => {
                if parts.len() < 3 {
                    println!(
                        "  {} Usage: inc <replica> <counter> [amount]",
                        "!".bright_red()
                    );
                    continue;
                }
                let amount: i64 = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);
                if let Some(replica) = replicas.get_mut(parts[1]) {
                    replica.increment(parts[2], amount);
                    step(&format!(
                        "{}.{} += {} → {}",
                        parts[1],
                        parts[2],
                        amount,
                        replica.value(parts[2])
                    ));
                } else {
                    println!("  {} Unknown replica '{}'", "!".bright_red(), parts[1]);
                }
            }

            "dec" | "-" => {
                if parts.len() < 3 {
                    println!(
                        "  {} Usage: dec <replica> <counter> [amount]",
                        "!".bright_red()
                    );
                    continue;
                }
                let amount: i64 = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(1);
                if let Some(replica) = replicas.get_mut(parts[1]) {
                    replica.decrement(parts[2], amount);
                    step(&format!(
                        "{}.{} -= {} → {}",
                        parts[1],
                        parts[2],
                        amount,
                        replica.value(parts[2])
                    ));
                } else {
                    println!("  {} Unknown replica '{}'", "!".bright_red(), parts[1]);
                }
            }

            "sync" => {
                if parts.len() < 3 {
                    println!("  {} Usage: sync <from> <to>", "!".bright_red());
                    continue;
                }
                let (from, to) = (parts[1], parts[2]);
                if !replicas.contains_key(from) {
                    println!("  {} Unknown replica '{}'", "!".bright_red(), from);
                    continue;
                }
                if !replicas.contains_key(to) {
                    println!("  {} Unknown replica '{}'", "!".bright_red(), to);
                    continue;
                }
                let snap = replicas[from].doc.clone_state();
                replicas.get_mut(to).unwrap().doc.merge(&snap);
                sync_arrow(from, to);
            }

            "syncall" => {
                if parts.len() < 2 {
                    println!("  {} Usage: syncall <name>", "!".bright_red());
                    continue;
                }
                let target = parts[1].to_string();
                if !replicas.contains_key(&target) {
                    println!("  {} Unknown replica '{}'", "!".bright_red(), target);
                    continue;
                }
                // Pull all others into target
                let other_snaps: Vec<JsonDoc> = replicas
                    .iter()
                    .filter(|(k, _)| **k != target)
                    .map(|(_, r)| r.doc.clone_state())
                    .collect();
                let t = replicas.get_mut(&target).unwrap();
                for snap in &other_snaps {
                    t.doc.merge(snap);
                }
                // Push target out to all others
                let target_snap = replicas[&target].doc.clone_state();
                let other_names: Vec<String> = replicas
                    .keys()
                    .filter(|k| **k != target)
                    .cloned()
                    .collect();
                for name in &other_names {
                    replicas.get_mut(name).unwrap().doc.merge(&target_snap);
                }
                step(&format!(
                    "'{}' synced bidirectionally with {} others",
                    target,
                    other_names.len()
                ));
            }

            "show" | "s" => {
                if parts.len() < 2 {
                    println!("  {} Usage: show <name>", "!".bright_red());
                    continue;
                }
                if let Some(replica) = replicas.get(parts[1]) {
                    show_replica(replica);
                } else {
                    println!("  {} Unknown replica '{}'", "!".bright_red(), parts[1]);
                }
            }

            "list" | "ls" => {
                if replicas.is_empty() {
                    println!("  {}", "(no replicas)".dimmed());
                } else {
                    let mut names: Vec<_> = replicas.keys().collect();
                    names.sort();
                    for name in names {
                        show_replica(&replicas[name]);
                    }
                }
            }

            "check" => {
                if parts.len() < 3 {
                    println!("  {} Usage: check <r1> <r2>", "!".bright_red());
                    continue;
                }
                let (n1, n2) = (parts[1], parts[2]);
                match (replicas.get(n1), replicas.get(n2)) {
                    (Some(r1), Some(r2)) => {
                        let mut all_names = r1.counter_names();
                        for n in r2.counter_names() {
                            if !all_names.contains(&n) {
                                all_names.push(n);
                            }
                        }
                        all_names.sort();

                        let mut converged = true;
                        for name in &all_names {
                            let v1 = r1.value(name);
                            let v2 = r2.value(name);
                            if v1 != v2 {
                                converged = false;
                                println!(
                                    "  {} '{}': {} has {}, {} has {}",
                                    "≠".bright_red(),
                                    name,
                                    n1,
                                    v1,
                                    n2,
                                    v2
                                );
                            } else {
                                println!(
                                    "  {} '{}': {} = {}",
                                    "=".bright_green(),
                                    name,
                                    v1,
                                    v2
                                );
                            }
                        }
                        convergence_result(converged);
                    }
                    _ => println!("  {} One or both replicas not found", "!".bright_red()),
                }
            }

            "detail" | "d" => {
                if parts.len() < 3 {
                    println!("  {} Usage: detail <replica> <counter>", "!".bright_red());
                    continue;
                }
                if let Some(replica) = replicas.get(parts[1]) {
                    let bd = replica.breakdown(parts[2]);
                    if bd.is_empty() {
                        println!("  {} Counter '{}' not found", "!".bright_yellow(), parts[2]);
                    } else {
                        println!("  {} breakdown for '{}':", parts[1].bright_yellow(), parts[2]);
                        for (rid, inc, dec) in &bd {
                            println!(
                                "    {} inc={} dec={} net={}",
                                rid.bright_white(),
                                inc.to_string().bright_green(),
                                dec.to_string().bright_red(),
                                (inc - dec).to_string().bright_cyan()
                            );
                        }
                        println!(
                            "    {} = {}",
                            "total".bold(),
                            replica.value(parts[2]).to_string().bold().bright_green()
                        );
                    }
                } else {
                    println!("  {} Unknown replica '{}'", "!".bright_red(), parts[1]);
                }
            }

            "quit" | "exit" | "q" => {
                println!("  {}", "Goodbye!".dimmed());
                break;
            }

            "help" | "h" | "?" => {
                println!("  replica <name> | inc <r> <c> [n] | dec <r> <c> [n]");
                println!("  sync <from> <to> | syncall <r> | show <r> | list");
                println!("  check <r1> <r2> | detail <r> <c> | quit");
            }

            other => {
                println!(
                    "  {} Unknown command '{}' — type 'help'",
                    "?".bright_yellow(),
                    other
                );
            }
        }
    }
}

// ─── Entry point ───────────────────────────────────────────────────────────

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Demo => run_demo(),
        Commands::Conflict => run_conflict(),
        Commands::Partition => run_partition(),
        Commands::Interactive => run_interactive(),
    }
}
