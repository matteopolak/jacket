#scope_module

// Length-limited prefix code construction and the canonical code assignment shared by the
// compressor's literal, distance and code-length alphabets.

// Fills `lengths[0..count)` with a prefix code no longer than `limit` bits for `freq`. Symbols
// with a zero frequency get length 0. The code is complete when two or more symbols are used.
build_code_lengths :: (freq: *u32, count: s64, limit: s64, lengths: *u8) {
    keys: [320] u64;
    n: s64;

    for i: 0..count - 1 {
        lengths[i] = 0;

        if freq[i] {
            weighted: u64 = freq[i];
            symbol := cast(u64) i;
            keys[n] = (weighted << 16) | symbol;
            n += 1;
        }
    }

    if n == 0 return;

    if n == 1 {
        lengths[keys[0] & 0xFFFF] = 1;
        return;
    }

    // Insertion sort, ascending by frequency (ties by symbol).
    for i: 1..n - 1 {
        key := keys[i];
        j := i - 1;

        while j >= 0 && keys[j] > key {
            keys[j + 1] = keys[j];
            j -= 1;
        }

        keys[j + 1] = key;
    }

    // Huffman merge with two queues: leaves in order, then internal nodes as they are made.
    weight: [640] u64;
    parent: [640] s32;
    depth: [640] u8;

    for i: 0..n - 1 weight[i] = keys[i] >> 16;

    leaf := 0;
    node := n;
    made := n;

    for 0..n - 2 {
        pair: [2] s64;

        for p: 0..1 {
            if leaf < n && (node >= made || weight[leaf] <= weight[node]) {
                pair[p] = leaf;
                leaf += 1;
            } else {
                pair[p] = node;
                node += 1;
            }
        }

        weight[made] = weight[pair[0]] + weight[pair[1]];
        parent[pair[0]] = cast(s32) made;
        parent[pair[1]] = cast(s32) made;
        made += 1;
    }

    root := made - 1;
    depth[root] = 0;

    for k: 0..root - 1 {
        index := root - 1 - k;
        depth[index] = depth[parent[index]] + 1;
    }

    deepest: s64;

    for i: 0..n - 1 {
        symbol := cast(s64) (keys[i] & 0xFFFF);
        lengths[symbol] = depth[i];
        deepest = max(deepest, cast(s64) depth[i]);
    }

    if deepest <= limit return;

    // Too deep: clamp, then repair the Kraft sum. Lengthen the rarest symbols until the code
    // fits, then shorten the commonest while there is room left.
    one: s64 = 1;
    space := one << limit;
    used: s64;

    for i: 0..n - 1 {
        symbol := cast(s64) (keys[i] & 0xFFFF);

        if lengths[symbol] > limit lengths[symbol] = cast(u8) limit;

        used += one << (limit - lengths[symbol]);
    }

    for i: 0..n - 1 {
        symbol := cast(s64) (keys[i] & 0xFFFF);

        while used > space && lengths[symbol] < limit {
            used -= one << (limit - lengths[symbol] - 1);
            lengths[symbol] += 1;
        }
    }

    changed := true;

    while changed {
        changed = false;

        for k: 0..n - 1 {
            symbol := cast(s64) (keys[n - 1 - k] & 0xFFFF);
            length := cast(s64) lengths[symbol];

            if length <= 1 continue;

            gain := one << (limit - length);

            if used + gain <= space {
                used += gain;
                lengths[symbol] -= 1;
                changed = true;
            }
        }
    }
}

// Canonical codes for `lengths`, bit-reversed for writing least significant bit first.
assign_codes :: (lengths: *u8, count: s64, codes: *u16) {
    counts: [16] u32;
    first: [16] u32;

    for i: 0..count - 1 counts[lengths[i]] += 1;

    counts[0] = 0;

    code: u32;

    for length: 1..15 {
        code = (code + counts[length - 1]) << 1;
        first[length] = code;
    }

    for i: 0..count - 1 {
        length := cast(u32) lengths[i];
        if length == 0 continue;

        codes[i] = cast(u16) reverse_bits(first[length], length);
        first[length] += 1;
    }
}
