-- Define alignment as a function: (n + 63) & ~63
-- In Lean, we can represent this using integer math for easier proof
def align_to_64 (n : Nat) : Nat :=
  ((n + 63) / 64) * 64

-- Define a theorem to prove result is always a multiple of 64
theorem aligned_is_multiple_of_64 (n : Nat) :
  ∃ k, align_to_64 n = 64 * k := by
  exists (n + 63) / 64
  unfold align_to_64
  rw [Nat.mul_comm]
