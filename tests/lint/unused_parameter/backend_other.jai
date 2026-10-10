// Not loaded by the cases: build-selected variants of `backend_step` and `slot_index`, which
// `good.jai` also defines. A definition here with the same parameter names, in the same order,
// is what keeps the unused parameters there quiet.
backend_step :: (count: int, scale: int) -> int {
    return count * scale;
}

fill_slots :: (slots: int) -> int {
    // Nested in a platform's procedure.
    slot_index :: (row: int, col: int) -> int {
        return row * 32 + col;
    }
    return slot_index(1, slots);
}
