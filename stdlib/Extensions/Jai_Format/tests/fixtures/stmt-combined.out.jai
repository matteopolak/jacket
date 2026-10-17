render :: (items: [] Item, mode: Mode) -> int {
    total := 0;
    limit := items.count;

    // clamp first
    if limit > 100 {
        limit = 100;
    }

    out, ok := open();
    if !ok return -1;
    defer close(out);

    for items {
        // skip hidden entries
        if it.hidden continue;
        total += it.size;
    }

    if mode == {
        case .FAST; return total;
        case .SLOW;
            // do the slow thing
            slow(items);
            total += 1;

        case .BOTH;
            fast(items);
            slow(items);

        case;
            return 0;
    }

    ok = flush(out);
    if !ok {
        return -2;
    }

    // done
    return total;
}
