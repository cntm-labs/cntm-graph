use cntm_graph::{MmapDeltaLog, init_shared_memory};
use std::env;
use std::thread::sleep;
use std::time::Duration;

fn main() {
    let args: Vec<String> = env::args().collect();
    let delta_path = if args.len() > 1 {
        &args[1]
    } else {
        "test_handshake.bin.delta"
    };

    println!("--- Isotime Mock Watcher ---");
    println!("Target: {}", delta_path);

    let mmap = match init_shared_memory(delta_path, 1024 * 1024) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: Could not map delta log: {}", e);
            eprintln!(
                "Hint: Run 'cargo test' first to generate a test delta log, or specify a path."
            );
            return;
        }
    };

    let log = MmapDeltaLog::new(mmap);

    // Start from the current tail to only see new events
    let mut last_processed_idx = unsafe { *log.tail_ptr };

    println!(
        "Connected. Initial head={}, tail={}. Waiting for events...",
        unsafe { *log.head_ptr },
        unsafe { *log.tail_ptr }
    );

    loop {
        let current_tail = unsafe { *log.tail_ptr };

        if current_tail != last_processed_idx {
            // Determine how many new events to read
            let mut current_idx = last_processed_idx;
            while current_idx != current_tail {
                let packet = unsafe { *log.data_ptr.add(current_idx as usize) };
                println!(
                    "[TIMESTAMP: {}] EVENT: {:?}",
                    packet.timestamp, packet.event
                );

                current_idx = (current_idx + 1) % log.capacity as u64;
            }
            last_processed_idx = current_tail;
        }

        sleep(Duration::from_millis(200));
    }
}
