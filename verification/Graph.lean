import verification.Memory

-- Model the offsets used in Rust src/lib.rs
-- IDs start at aligned 8, len = cap * 8
-- Types start at aligned (ids_end), len = cap * 2
-- Weights start at aligned (types_end), len = cap * 4
def node_layout (capacity : Nat) : List Segment :=
  let ids := Segment.mk (align_to_64 8) (capacity * 8)
  let types := Segment.mk (align_to_64 (ids.start + ids.len)) (capacity * 2)
  let weights := Segment.mk (align_to_64 (types.start + types.len)) (capacity * 4)
  [ids, types, weights]

-- Theorem: IDs and Types segments never overlap
theorem ids_types_non_overlap (cap : Nat) :
  let layout := node_layout cap
  non_overlapping (layout.get! 0) (layout.get! 1) := by
  unfold node_layout non_overlapping
  simp
  apply align_to_64_ge
