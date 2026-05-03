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

structure Segment where
  start : Nat
  len : Nat

def non_overlapping (s1 s2 : Segment) : Prop :=
  s1.start + s1.len <= s2.start ∨ s2.start + s2.len <= s1.start

-- Theorem to prove: align_to_64 n >= n
theorem align_to_64_ge (n : Nat) : align_to_64 n >= n := by
  unfold align_to_64
  cases n with
  | zero => simp
  | succ n' =>
    have h1 : n + 63 >= 64 := by
      rw [Nat.add_comm]
      apply Nat.le_add_right
    have h2 : (n + 63) / 64 >= 1 := by
      apply Nat.div_le_div_right h1
    exact Nat.le_mul_of_pos_left ((n + 63) / 64) (by exact Nat.zero_lt_succ 63)
