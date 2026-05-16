use cntm_graph::GraphStore;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <graph_file_path>", args[0]);
        process::exit(1);
    }

    let path = &args[1];

    // Attempt to load the graph store. We assume capacities are known or we use defaults.
    // For analysis, we don't necessarily need to know the exact original capacities
    // but GraphStore::new requires them.
    // In a real scenario, we might store capacities in a header.
    // For now, let's assume standard test capacities or try to infer.
    let node_cap = 1_000_000;
    let edge_cap = 10_000_000;

    println!("Loading GraphStore from: {}", path);
    let mut store = match GraphStore::new(path, node_cap, edge_cap) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to load GraphStore: {}", e);
            process::exit(1);
        }
    };

    let report = store.analyze_fragmentation();
    println!("\n--- Memory Fragmentation Report ---");
    println!("Arena Size:        {} bytes", report.total_arena_size);
    println!("Current Offset:    {} bytes", report.current_offset);
    println!("Alive Bytes:       {} bytes", report.alive_bytes);
    println!("Dead Bytes:        {} bytes", report.dead_bytes);
    println!(
        "Fragmentation:     {:.2}%",
        report.fragmentation_ratio * 100.0
    );
    println!("-----------------------------------\n");

    if report.fragmentation_ratio > 0.10 {
        println!("Fragmentation is above 10%. Triggering compaction...");
        match store.compact_metadata() {
            Ok(_) => {
                println!("Compaction successful.");
                let new_report = store.analyze_fragmentation();
                println!(
                    "New Fragmentation: {:.2}%",
                    new_report.fragmentation_ratio * 100.0
                );
            }
            Err(e) => {
                eprintln!("Compaction failed: {}", e);
            }
        }
    } else {
        println!("Fragmentation is low. No compaction needed.");
    }
}
