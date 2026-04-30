from std.ffi import external_call

def main() raises:
    print("Mojo Reader: Starting Full E2E Verification with Linked C Helper...")

    var path = "/home/mrbt/Desktop/workspaces/mlops/repositories/cntm-graph/e2e_test_graph.bin"
    var node_cap = 1000
    var edge_cap = 1000

    def align_64(val: Int) -> Int:
        return (val + 63) & ~63

    var ids_offset = align_64(8)
    var type_ids_offset = align_64(ids_offset + (node_cap * 8))
    var states_offset = align_64(type_ids_offset + (node_cap * 2))
    var weights_offset = align_64(states_offset + node_cap)
    var timestamps_offset = align_64(weights_offset + (node_cap * 4))
    var ext_offsets_offset = align_64(timestamps_offset + (node_cap * 8))
    var nodes_size = align_64(ext_offsets_offset + (node_cap * 4))

    var src_offset = align_64(8)
    var tgt_offset = align_64(src_offset + (edge_cap * 4))
    var types_offset = align_64(tgt_offset + (edge_cap * 4))
    var edge_weights_offset = align_64(types_offset + (edge_cap * 2))
    var edges_size = align_64(edge_weights_offset + (edge_cap * 4))

    var total_size = nodes_size + edges_size

    var ptr = external_call["map_graph", Int](path.unsafe_ptr(), total_size)
    if ptr == 0:
        print("Mojo Reader Error: Failed to map graph")
        return

    print("Mojo Reader: [SUCCESS] Memory Mapped at", ptr)

    var count = external_call["read_u64", UInt64](ptr, 0)
    print("Mojo Reader: Node Count =", count)

    # Node 0: ID 100, Type 10, Weight 0.5
    var n0_id = external_call["read_u64", UInt64](ptr, ids_offset)
    var n0_type = external_call["read_u16", UInt16](ptr, type_ids_offset)
    var n0_weight = external_call["read_f32", Float32](ptr, weights_offset)
    print("Node 0: ID", n0_id, "Type", n0_type, "Weight", n0_weight)

    # Node 2: ID 300, Type 10, Weight 0.9 (Index 2)
    var n2_id = external_call["read_u64", UInt64](ptr, ids_offset + 16)
    var n2_type = external_call["read_u16", UInt16](ptr, type_ids_offset + 4)
    var n2_weight = external_call["read_f32", Float32](ptr, weights_offset + 8)
    print("Node 2: ID", n2_id, "Type", n2_type, "Weight", n2_weight)

    # Edge data
    var edge_base = ptr + nodes_size
    var edge_count = external_call["read_u64", UInt64](edge_base, 0)
    print("Mojo Reader: Edge Count =", edge_count)

    # Edge 1: Src 1 -> Tgt 2, Type 2, Weight 0.2 (Index 1)
    var e1_src = external_call["read_u32", UInt32](edge_base, src_offset + 4)
    var e1_tgt = external_call["read_u32", UInt32](edge_base, tgt_offset + 4)
    var e1_type = external_call["read_u16", UInt16](edge_base, types_offset + 2)
    var e1_weight = external_call["read_f32", Float32](edge_base, edge_weights_offset + 4)
    print("Edge 1: Src", e1_src, "Tgt", e1_tgt, "Type", e1_type, "Weight", e1_weight)

    if count == 3 and n0_id == 100 and n2_weight == 0.9 and edge_count == 2 and e1_src == 1:
        print("Mojo Reader: [SUCCESS] Full Zero-copy verification passed!")
    else:
        print("Mojo Reader Error: Validation failed")

    _ = external_call["unmap_graph", NoneType](ptr, total_size)
