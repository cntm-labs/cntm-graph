use cntm_graph::GraphStore;
use std::fs;

fn main() -> std::io::Result<()> {
    let path = "e2e_test_graph.bin";
    // Ensure clean state
    if fs::metadata(path).is_ok() {
        fs::remove_file(path)?;
    }

    let node_cap = 1000;
    let edge_cap = 1000;

    println!("Rust Writer: Initializing GraphStore at {}...", path);
    let mut store = GraphStore::new(path, node_cap, edge_cap)?;

    println!("Rust Writer: Adding test nodes...");
    // Node 0: ID 100, Type 10, Weight 0.5
    store.nodes.add_node(100, 10, 0.5);
    // Node 1: ID 200, Type 20, Weight 0.8
    store.nodes.add_node(200, 20, 0.8);
    // Node 2: ID 300, Type 10, Weight 0.9 (Best for type 10)
    store.nodes.add_node(300, 10, 0.9);

    println!("Rust Writer: Adding test edges...");
    // Edge 0: Src 0 -> Tgt 1, Type 1, Weight 0.1
    store.edges.add_edge(0, 1, 1, 0.1);
    // Edge 1: Src 1 -> Tgt 2, Type 2, Weight 0.2
    store.edges.add_edge(1, 2, 2, 0.2);

    println!("Rust Writer: Successfully wrote {} nodes and {} edges.", store.nodes.count, store.edges.count);
    Ok(())
}
