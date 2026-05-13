use cntm_graph::GraphStore;
use std::fs;

fn main() {
    let path = "canary_test.bin";
    let node_cap = 16;
    let edge_cap = 16;

    // Clean up previous run files if they exist
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(format!("{}.meta", path));
    let _ = fs::remove_file(format!("{}.delta", path));

    println!("Initializing GraphStore with capacity {}...", node_cap);
    let store = GraphStore::new(path, node_cap, edge_cap).expect("Failed to create GraphStore");

    // Initially, canaries should be intact
    println!(
        "Initial canary verification: {}",
        if store.verify_canaries() {
            "PASS"
        } else {
            "FAIL"
        }
    );
    assert!(
        store.verify_canaries(),
        "Canaries should be valid after initialization"
    );

    println!("Simulating memory corruption (Canary Breach)...");

    // Access the node weights pointer (DOD segment)
    // weights_ptr is *mut f32. For capacity 16, weights take 16 * 4 = 64 bytes.
    // The GUARD_SIZE block starts immediately after these 64 bytes.
    let weights_ptr = store.nodes.weights_ptr;

    unsafe {
        // Purposely write past the end of the weights segment into the adjacent guard block.
        // We cast to *mut u64 to corrupt the first 8 bytes of the 64-byte guard.
        let breach_ptr = weights_ptr.add(node_cap) as *mut u64;

        println!("Writing corruption at pointer: {:?}", breach_ptr);
        breach_ptr.write(0xBAD0000000000BAD);
    }

    // Verify detection
    if !store.verify_canaries() {
        println!("[SUCCESS] Canary Breach Detected!");
    } else {
        println!("[FAILURE] Canary Breach NOT Detected!");

        // Final cleanup before exiting with error
        let _ = fs::remove_file(path);
        let _ = fs::remove_file(format!("{}.meta", path));
        let _ = fs::remove_file(format!("{}.delta", path));
        std::process::exit(1);
    }

    // Clean up
    let _ = fs::remove_file(path);
    let _ = fs::remove_file(format!("{}.meta", path));
    let _ = fs::remove_file(format!("{}.delta", path));
}
