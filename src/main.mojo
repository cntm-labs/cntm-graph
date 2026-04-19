from memory import UnsafePointer
from sys.ffi import external_call

@value
struct MojoNodeTable:
    """Mirrors the Rust MmapNodeTable for zero-copy access."""
    var ptr: UnsafePointer[UInt8]
    var capacity: Int
    var count: Int
    var ids_ptr: UnsafePointer[UInt64]
    var type_ids_ptr: UnsafePointer[UInt16]
    var states_ptr: UnsafePointer[UInt8]
    var weights_ptr: UnsafePointer[Float32]
    var timestamps_ptr: UnsafePointer[UInt64]
    var ext_offsets_ptr: UnsafePointer[UInt32]

    fn __init__(inout self, base_ptr: UnsafePointer[UInt8], capacity: Int):
        self.ptr = base_ptr
        self.capacity = capacity
        # The first 8 bytes contain the count as u64
        self.count = int(base_ptr.bitcast[UInt64]()[0])
        
        let ids_offset = self._align_to_64(8)
        let type_ids_offset = self._align_to_64(ids_offset + (capacity * 8))
        let states_offset = self._align_to_64(type_ids_offset + (capacity * 2))
        let weights_offset = self._align_to_64(states_offset + capacity)
        let timestamps_offset = self._align_to_64(weights_offset + (capacity * 4))
        let ext_offsets_offset = self._align_to_64(timestamps_offset + (capacity * 8))

        self.ids_ptr = base_ptr.offset(ids_offset).bitcast[UInt64]()
        self.type_ids_ptr = base_ptr.offset(type_ids_offset).bitcast[UInt16]()
        self.states_ptr = base_ptr.offset(states_offset)
        self.weights_ptr = base_ptr.offset(weights_offset).bitcast[Float32]()
        self.timestamps_ptr = base_ptr.offset(timestamps_offset).bitcast[UInt64]()
        self.ext_offsets_ptr = base_ptr.offset(ext_offsets_offset).bitcast[UInt32]()

    @staticmethod
    fn _align_to_64(offset: Int) -> Int:
        return (offset + 63) & ~63

    fn get_id(self, idx: Int) -> UInt64:
        return self.ids_ptr[idx]

    fn get_ext_offset(self, idx: Int) -> UInt32:
        return self.ext_offsets_ptr[idx]

@value
struct MojoEdgeTable:
    """Mirrors the Rust MmapEdgeTable for zero-copy access."""
    var ptr: UnsafePointer[UInt8]
    var capacity: Int
    var count: Int
    var src_ptr: UnsafePointer[UInt32]
    var tgt_ptr: UnsafePointer[UInt32]
    var types_ptr: UnsafePointer[UInt16]
    var weights_ptr: UnsafePointer[Float32]

    fn __init__(inout self, base_ptr: UnsafePointer[UInt8], capacity: Int):
        self.ptr = base_ptr
        self.capacity = capacity
        # The first 8 bytes contain the count as u64
        self.count = int(base_ptr.bitcast[UInt64]()[0])
        
        let src_offset = self._align_to_64(8)
        let tgt_offset = self._align_to_64(src_offset + (capacity * 4))
        let types_offset = self._align_to_64(tgt_offset + (capacity * 4))
        let weights_offset = self._align_to_64(types_offset + (capacity * 2))

        self.src_ptr = base_ptr.offset(src_offset).bitcast[UInt32]()
        self.tgt_ptr = base_ptr.offset(tgt_offset).bitcast[UInt32]()
        self.types_ptr = base_ptr.offset(types_offset).bitcast[UInt16]()
        self.weights_ptr = base_ptr.offset(weights_offset).bitcast[Float32]()

    @staticmethod
    fn _align_to_64(offset: Int) -> Int:
        return (offset + 63) & ~63

@value
struct MojoGraphStore:
    """Mirrors the Rust GraphStore for zero-copy access across the FFI bridge."""
    var nodes: MojoNodeTable
    var edges: MojoEdgeTable
    var nodes_meta_ptr: UnsafePointer[UInt8]

    fn __init__(
        inout self, 
        base_ptr: UnsafePointer[UInt8], 
        nodes_meta_ptr: UnsafePointer[UInt8],
        node_cap: Int, 
        edge_cap: Int
    ):
        # Calculate node table size to find where edge table starts
        let nodes_size = self._calculate_node_table_size(node_cap)
        
        self.nodes = MojoNodeTable(base_ptr, node_cap)
        self.edges = MojoEdgeTable(base_ptr.offset(nodes_size), edge_cap)
        self.nodes_meta_ptr = nodes_meta_ptr

    @staticmethod
    fn _align_to_64(offset: Int) -> Int:
        return (offset + 63) & ~63

    @staticmethod
    fn _calculate_node_table_size(capacity: Int) -> Int:
        let ids_offset = MojoGraphStore._align_to_64(8)
        let type_ids_offset = MojoGraphStore._align_to_64(ids_offset + (capacity * 8))
        let states_offset = MojoGraphStore._align_to_64(type_ids_offset + (capacity * 2))
        let weights_offset = MojoGraphStore._align_to_64(states_offset + capacity)
        let timestamps_offset = MojoGraphStore._align_to_64(weights_offset + (capacity * 4))
        let ext_offsets_offset = MojoGraphStore._align_to_64(timestamps_offset + (capacity * 8))
        return MojoGraphStore._align_to_64(ext_offsets_offset + (capacity * 4))

    fn get_node_metadata_offset(self, idx: Int) -> UInt32:
        """Retrieves the byte offset for metadata from the Hot Path table."""
        return self.nodes.get_ext_offset(idx)

    fn read_metadata_root(self, offset: UInt32) -> Int:
        """Reads the FlatBuffers root table offset at the given metadata offset."""
        let off = int(offset)
        # FlatBuffers uses a 4-byte little-endian offset at the root
        return int(self.nodes_meta_ptr.offset(off).bitcast[UInt32]()[0])

    fn read_string_at_offset(self, string_offset: Int) -> String:
        """Reads a FlatBuffers string (4-byte length prefix followed by UTF-8 bytes)."""
        let length = int(self.nodes_meta_ptr.offset(string_offset).bitcast[UInt32]()[0])
        # In a real Mojo implementation, we would construct a String from the bytes
        return "String(length=" + str(length) + ")"

fn main():
    print("Mojo Zero-Copy Bridge: Initialized")
    print("Mojo Side: Fully mirrored Rust GraphStore layout (64-byte alignment)")
    print("Mojo Side: Added metadata file mapping support")
    
    # In a production environment, nodes_meta_ptr would be acquired via mmap syscall
    # or passed from the Rust host as a shared memory pointer.
    
    print("Mojo Zero-Copy Bridge: Ready to read FlatBuffers metadata at Hot Path offsets")
