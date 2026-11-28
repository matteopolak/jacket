// Returns the number of members of `set`.
cardinality :: (set: Int_Set) -> u64 {
    count: u64;

    for word: set.words {
        remaining := word;
        while remaining != 0 {
            remaining &= remaining - 1;
            count += 1;
        }
    }

    return count;
}

// Returns whether `i` is a member of `set`; a negative or out-of-range `i` gives `false`.
is_set :: (set: Int_Set, i: int) -> bool {
    if i < 0 || i / Int_Set.BPW >= set.words.count return false;
    return (set.words[i / Int_Set.BPW] & ((cast(u32) 1) << (i % Int_Set.BPW))) != 0;
}
