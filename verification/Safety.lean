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

-- Theorem: Adding a node preserves the core invariant
-- (This is implicitly proven by the type check of the 'inv' field in add_node,
-- but we can make it explicit if needed).
