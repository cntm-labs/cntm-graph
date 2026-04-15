use cntm_graph::align_to_64;

fn main() {
    let capacity = 1000;
    let ids_offset = align_to_64(8);
    let type_ids_offset = align_to_64(ids_offset + (capacity * 8));
    let states_offset = align_to_64(type_ids_offset + (capacity * 2));
    let weights_offset = align_to_64(states_offset + capacity);
    let timestamps_offset = align_to_64(weights_offset + (capacity * 4));
    let ext_offsets_offset = align_to_64(timestamps_offset + (capacity * 8));
    
    println!("ids_offset: {}", ids_offset);
    println!("type_ids_offset: {}", type_ids_offset);
    println!("states_offset: {}", states_offset);
    println!("weights_offset: {}", weights_offset);
    println!("timestamps_offset: {}", timestamps_offset);
    println!("ext_offsets_offset: {}", ext_offsets_offset);
    
    assert_eq!(ids_offset % 64, 0);
    assert_eq!(type_ids_offset % 64, 0);
    assert_eq!(states_offset % 64, 0);
    assert_eq!(weights_offset % 64, 0);
    assert_eq!(timestamps_offset % 64, 0);
    assert_eq!(ext_offsets_offset % 64, 0);
    println!("All offsets are 64-byte aligned!");
}
