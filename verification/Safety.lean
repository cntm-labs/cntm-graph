import verification.Memory

structure GraphState where
  capacity : Nat
  node_count : Nat
  -- Invariant: node_count must never exceed capacity
  inv : node_count <= capacity

-- Define a mutation: adding a node
-- Returns Some new state if there is room, None otherwise
def add_node (s : GraphState) : Option GraphState :=
  if h : s.node_count < s.capacity then
    Some {
      capacity := s.capacity,
      node_count := s.node_count + 1,
      inv := Nat.le_of_lt_succ (Nat.add_one_le_iff.mpr (Nat.succ_le_of_lt h))
    }
  else
    None

theorem addr_in_segment_bounds (s : Segment) (i : Nat) (h : s.in_bounds i) :
  s.start <= s.addr i ∧ s.addr i < s.start + s.len := by
  constructor
  · unfold Segment.addr
    apply Nat.le_add_right
  · unfold Segment.addr
    unfold Segment.in_bounds at h
    apply Nat.add_lt_add_left h

/--
Theorem: Isolation of Data from Guards.
If a data segment and a guard segment are non-overlapping,
then any valid index into the data segment will never point into the guard.
-/
theorem isolation_from_guard (data_seg guard_seg : Segment) (i : Nat)
  (h_non_overlap : non_overlapping data_seg guard_seg)
  (h_in_bounds : data_seg.in_bounds i) :
  data_seg.addr i < guard_seg.start ∨ data_seg.addr i >= guard_seg.start + guard_seg.len := by
  have h_bounds := addr_in_segment_bounds data_seg i h_in_bounds
  cases h_non_overlap with
  | inl h1 =>
    -- data_seg.start + data_seg.len <= guard_seg.start
    apply Or.inl
    exact Nat.lt_of_lt_of_le h_bounds.2 h1
  | inr h2 =>
    -- guard_seg.start + guard_seg.len <= data_seg.start
    apply Or.inr
    exact Nat.le_trans h2 h_bounds.1
