get_line_mapping_for_address :: (info: Line_Info, address: u64) -> Line_Mapping, success: bool {
    empty: Line_Mapping;
    if !line_sequences_valid(info) return empty, false;

    found := false;
    result: Line_Mapping;

    for sequence: info.sequences {
        if sequence.addresses.count < 2 || address < sequence.addresses[0] || address >= sequence.addresses[sequence.addresses.count - 1] continue;

        if found {
            log_error("Debug_Info: overlapping line sequences cover the queried address\n");
            return empty, false;
        }

        low: s64;
        high := sequence.addresses.count;

        while low < high {
            middle := low + (high - low) / 2;
            if sequence.addresses[middle] <= address low = middle + 1;
            else high = middle;
        }

        result = sequence.mappings[low - 1];
        found = true;
    }

    return result, found;
}

get_line_mappings_for_address_range :: (info: Line_Info, start: u64, end_exclusive: u64) -> [] Line_Mapping, success: bool {
    if end_exclusive < start {
        log_error("Debug_Info: reversed line address range\n");
        return .{}, false;
    }

    if !line_sequences_valid(info) return .{}, false;
    if start == end_exclusive return .{}, true;

    for sequence, index: info.sequences {
        if sequence.addresses.count < 2 continue;

        first := max(start, sequence.addresses[0]);
        last := min(end_exclusive, sequence.addresses[sequence.addresses.count - 1]);

        if first >= last continue;

        for previous: 0..index - 1 {
            other := info.sequences[previous];
            if other.addresses.count < 2 continue;

            if max(first, other.addresses[0]) < min(last, other.addresses[other.addresses.count - 1]) {
                log_error("Debug_Info: overlapping line sequences cover the queried range\n");
                return .{}, false;
            }
        }
    }

    rows: [..] Line_Mapping;

    for sequence: info.sequences {
        for index: 0..sequence.addresses.count - 2 {
            if max(start, sequence.addresses[index]) < min(end_exclusive, sequence.addresses[index + 1]) array_add(*rows, sequence.mappings[index]);
        }
    }

    return .{ rows.count, rows.data }, true;
}

#scope_file
#import "Basic";

line_sequences_valid :: (info: Line_Info) -> bool {
    for sequence: info.sequences {
        count := sequence.addresses.count;
        if count != sequence.mappings.count {
            log_error("Debug_Info: line sequence address/mapping counts differ\n");
            return false;
        }

        if count == 0 continue;

        if !sequence.mappings[count - 1].is_end {
            log_error("Debug_Info: line sequence has no terminating row\n");
            return false;
        }

        for index: 0..count - 2 {
            if sequence.mappings[index].is_end || sequence.addresses[index] > sequence.addresses[index + 1] {
                log_error("Debug_Info: invalid line sequence order or premature terminating row\n");
                return false;
            }
        }
    }

    return true;
}
