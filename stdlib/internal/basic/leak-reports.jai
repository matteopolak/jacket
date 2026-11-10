allocations_before_this_call_will_not_be_considered_leaks :: () {
    #if MEMORY_DEBUGGER {
        debug_acquire();
        debug_baseline = debug_ordinal;
        debug_unlock();
    }
}

#if MEMORY_DEBUGGER {
    this_allocation_is_not_a_leak :: (memory: *void) #expand #no_debug {
        mark_debug_allocation(memory, false);
    }

    this_allocation_leaks_todo :: (memory: *void) #expand #no_debug {
        mark_debug_allocation(memory, false);
    }

    this_allocation_does_not_propagate_leak_ownership :: (memory: *void) #expand #no_debug {
        mark_debug_allocation(memory, true);
    }
} else {
    this_allocation_is_not_a_leak :: (memory: *void) #expand #no_debug {}
    this_allocation_leaks_todo :: (memory: *void) #expand #no_debug {}
    this_allocation_does_not_propagate_leak_ownership :: (memory: *void) #expand #no_debug {}
}

add_allocation_to_table :: (memory: *void, size: s64, allocator: Allocator, trace: *Stack_Trace_Node) {
    assert(memory != null && size >= 0);
    debug_record(allocator, memory, size, false, trace);
}

do_free_bookkeeping :: (allocation: *Allocation) {
    assert(allocation != null);
    debug_release(allocation.owner, allocation.memory, allocation.heap, allocation.block);
}

add_allocation_this_frame :: (a: *Allocation, size: s64) {
    assert(a != null && size >= 0);
    debug_acquire();
    a.frame_allocations += 1;
    a.frame_bytes += size;
    frame_allocations += 1;
    frame_allocated_bytes += cast(u64) size;
    visualization_note_locked(a, false, size);
    debug_unlock();
}

add_free_this_frame :: (a: *Allocation, size: s64) {
    assert(a != null && size >= 0);
    debug_acquire();
    frame_frees += 1;
    frame_freed_bytes += cast(u64) size;
    visualization_note_locked(a, true, size);
    debug_unlock();
}

make_leak_report :: (options: Leak_Report_Options = .{}, for_visualizer := false) -> Leak_Report {
    debug_acquire();
    defer debug_unlock();

    return build_leak_report_locked(options, for_visualizer);
}

#scope_module
build_leak_report_locked :: (options: Leak_Report_Options, for_visualizer: bool) -> Leak_Report { // jailint: allow(cognitive_complexity)
    // The debugger's lock is held and the report's pool takes blocks: they are not for the debugger to record.
    push_context, defer_pop;
    context.inside_memory_debugger = true;
    report: Leak_Report;

    RP.set_allocators(*report.pool, context.allocator);
    count: s64;
    record := debug_records;

    while record {
        if !record.heap count += 1;
        record = record.next;
    }

    if count == 0 return report;

    table_pool := cast(*RP.Pool) report_memory(*report, size_of(RP.Pool));
    table_pool.* = .{};
    RP.set_allocators(table_pool, context.allocator);
    report.leak_combiner_table.allocator = .{ RP.pool_allocator_proc, table_pool };
    work := ([] Report_Allocation).{ count, cast(*Report_Allocation) report_memory(*report, count * size_of(Report_Allocation)) };

    memset(work.data, 0, count * size_of(Report_Allocation));
    RT.table_ensure_space(*report.leak_combiner_table, count);
    record = debug_records;
    index: s64;

    while record {
        if !record.heap {
            work[index].allocation = record;
            RT.table_add(*report.leak_combiner_table, report_key(record), .{});
            index += 1;
        }

        record = record.next;
    }

    scratch: RP.Pool;

    RP.set_allocators(*scratch, context.allocator);
    defer RP.release(*scratch);

    // Stable ordinal order makes multiple candidate owners deterministic.
    sort_by_ordinal(work, *scratch);

    for *item: work {
        item.entry = RT.table_find_pointer(*report.leak_combiner_table, report_key(item.allocation));
        a := item.allocation;
        item.entry.exempt = a.exempt || (!for_visualizer && a.ordinal <= debug_baseline);
        item.entry.reachable = item.entry.exempt;
    }

    addresses := build_address_index(work, *scratch);

    if !for_visualizer && options.scan_global_memory_to_remove_leaks {
        runtime := LC.get_runtime_info();
        assert(runtime.global_data_info != null, "compiler runtime global-data metadata is required for leak scanning");
        for segment: runtime.global_data_info.segment_info mark_range(segment.data.data, segment.data.count, *addresses, root = true);
    }

    changed := !for_visualizer;
    while changed {
        changed = false;

        for *item: work {
            if !item.entry.reachable || item.scanned continue;

            item.scanned = true;
            changed = true;
            if item.allocation.stops_ownership continue;

            mark_range(cast(*u8) item.allocation.memory, item.allocation.size, *addresses);
        }
    }

    // Only the allocations marked or predating the baseline are counted; what they reach is merely hidden.
    for item: work if item.entry.exempt {
        report.marked_as_non_leak_allocations += 1;
        report.marked_as_non_leak_bytes += item.allocation.size;
    }

    if !for_visualizer && options.scan_leaks_to_combine {
        for item: work {
            if item.entry.reachable && !for_visualizer continue;

            a := item.allocation;
            if a.stops_ownership continue;

            for offset: 0..a.size - size_of(*void) {
                pointer: *void;

                memcpy(*pointer, cast(*u8) a.memory + offset, size_of(*void));
                child := containing_allocation(pointer, *addresses);
                if !child || child.allocation == a || child.entry.owner continue;
                if child.entry.exempt && !for_visualizer continue;

                parent := item.entry;
                cycle := false;

                while parent.owner {
                    if parent.owner == report_key(child.allocation) {
                        cycle = true;
                        break;
                    }

                    parent = RT.table_find_pointer(*report.leak_combiner_table, parent.owner);
                }

                if !cycle child.entry.owner = report_key(a);
            }
        }
    }

    summaries := cast(**Leak_Summary) report_memory(*report, count * size_of(*Leak_Summary));

    report.sorted_summaries = .{ 0, summaries };
    groups := cast(**Group_Info) report_memory(*report, options.group_by_these_calls.count * size_of(*Group_Info));

    memset(groups, 0, options.group_by_these_calls.count * size_of(*Group_Info));

    for item: work {
        if (item.entry.reachable && !for_visualizer) || item.entry.owner continue;

        a := item.allocation;
        display_trace := pool_trace_copy(*report, a.trace, options);
        summary: *Leak_Summary;

        for existing: report.sorted_summaries if traces_equal(existing.alloc_site_trace, display_trace) {
            summary = existing;
            break;
        }

        if !summary {
            summary = cast(*Leak_Summary) report_memory(*report, size_of(Leak_Summary));
            summary.* = .{};
            summary.alloc_site_trace = display_trace;
            summary.allocator_proc = a.owner.proc;
            report.sorted_summaries.data[report.sorted_summaries.count] = summary;
            report.sorted_summaries.count += 1;

            for procedure, group_index: options.group_by_these_calls {
                one := ([] Procedure_To_Elide).{ 1, *procedure };
                if !trace_matches(a.trace, one) continue;

                if !groups[group_index] {
                    group := cast(*Group_Info) report_memory(*report, size_of(Group_Info));
                    group.* = .{};
                    group.procedure = .{ pool_string(*report, procedure.procedure_name), pool_string(*report, procedure.module_name) };
                    groups[group_index] = group;
                }

                summary.group_info = groups[group_index];
                break;
            }
        }

        summary.count += 1;
        if summary.allocator_proc != a.owner.proc summary.allocator_proc = null;

        summary.bytes += a.size;
        summary.allocations_since_last_visualizer_update += a.frame_allocations;
        summary.bytes_allocated_since_last_visualizer_update += a.frame_bytes;
        item.entry.summary = summary;
    }

    for item: work {
        if !item.entry.owner continue;

        parent := item.entry;
        while parent.owner parent = RT.table_find_pointer(*report.leak_combiner_table, parent.owner);

        item.entry.summary = parent.summary;
        parent.summary.child_count += 1;
        parent.summary.child_bytes += item.allocation.size;
    }

    build_groups(*report, groups, options.group_by_these_calls.count);

    for i: 1..report.sorted_summaries.count - 1 {
        value := report.sorted_summaries[i];
        weight := value.bytes + value.child_bytes;
        j := i;

        while j > 0 && report.sorted_summaries[j - 1].bytes + report.sorted_summaries[j - 1].child_bytes < weight {
            report.sorted_summaries[j] = report.sorted_summaries[j - 1];
            j -= 1;
        }

        report.sorted_summaries[j] = value;
    }

    return report;
}

#scope_export
deinit :: (report: *Leak_Report) {
    table_pool := ifx report.leak_combiner_table.allocator.proc == RP.pool_allocator_proc then cast(*RP.Pool) report.leak_combiner_table.allocator.data else null;

    RT.deinit(*report.leak_combiner_table);
    if table_pool RP.release(table_pool);

    RP.release(*report.pool);
    report.* = .{};
}

log_leak_report :: (report: Leak_Report, options: Leak_Report_Options = .{}) {
    count: s64;
    bytes: s64;

    for summary: report.sorted_summaries {
        count += summary.count + summary.child_count;
        bytes += summary.bytes + summary.child_bytes;
    }

    for summary: report.sorted_summaries {
        if summary.group_info log("Group: %\n", summary.group_info.procedure.procedure_name);
        else for procedure: options.group_by_these_calls {
            one := ([] Procedure_To_Elide).{ 1, *procedure };
            if trace_matches(summary.alloc_site_trace, one) {
                log("Group: %\n", procedure.procedure_name);
                break;
            }
        }

        log_error("  % allocations / % bytes; % children / % bytes\n", summary.count, summary.bytes, summary.child_count, summary.child_bytes);

        if summary.alloc_site_trace {
            begin, end := trace_bounds(summary.alloc_site_trace, options);
            for index: begin..end - 1 {
                node := summary.alloc_site_trace.nodes[index];
                if node.info log_error("    '%' at %:%\n", node.info.name, node.info.location.fully_pathed_filename, node.line_number);
            }
        }
    }

    log_error("Memory leaks: % allocations / % bytes\n", count, bytes);
    log("Marked non-leaks: % allocations / % bytes\n", report.marked_as_non_leak_allocations, report.marked_as_non_leak_bytes);
}

report_memory_leaks :: (options: Leak_Report_Options = .{}) {
    report := make_leak_report(options);
    defer deinit(*report);
    log_leak_report(report, options);
}

#scope_module
mark_debug_allocation :: (memory: *void, stop_ownership: bool) {
    if !memory return;

    debug_acquire();
    record := debug_records;
    while record {
        if !record.heap && !record.block && record.memory == memory break;
        record = record.next;
    }

    if record {
        if stop_ownership record.stops_ownership = true;
        else record.exempt = true;
    }

    debug_unlock();

    if !record && !stop_ownership {
        log_error("memory debugger: unknown allocation marked as non-leak");
        terminate_debug_misuse(1);
    }
}

#scope_file
RP :: #import "Pool";
RT :: #import "Hash_Table";
LC :: #import "Compiler";

leak_libc :: #system_library "libc"; @native(external)
terminate_debug_misuse :: (status: s32) #foreign leak_libc "exit"; @Exits

Report_Allocation :: struct {
    allocation: *Allocation;
    entry: *Leak_Combiner_Record;
    scanned: bool;
}

report_memory :: (report: *Leak_Report, size: s64) -> *void {
    return pool_memory(*report.pool, size);
}

pool_memory :: (pool: *RP.Pool, size: s64) -> *void {
    if size == 0 return null;

    memory := RP.get(pool, size);
    if !memory {
        context.assertion_failed(#caller_location, "memory debugger: report allocation failed");
        debug_break();
    }

    return memory;
}

pool_string :: (report: *Leak_Report, value: string) -> string {
    data := cast(*u8) report_memory(report, value.count);
    memcpy(data, value.data, value.count);
    return .{ value.count, data };
}

pool_trace_copy :: (report: *Leak_Report, trace: *Packed_Stack_Trace, options: Leak_Report_Options) -> *Packed_Stack_Trace {
    if !trace return null;

    begin, end := trace_bounds(trace, options);
    count := end - begin;
    result := cast(*Packed_Stack_Trace) report_memory(report, size_of(Packed_Stack_Trace));
    result.nodes = .{ count, cast(*Stack_Trace_Node) report_memory(report, count * size_of(Stack_Trace_Node)) };
    memcpy(result.nodes.data, trace.nodes.data + begin, count * size_of(Stack_Trace_Node));
    for *node, index: result.nodes node.next = ifx index + 1 < result.nodes.count then result.nodes.data + index + 1 else null;

    return result;
}

trace_bounds :: (trace: *Packed_Stack_Trace, options: Leak_Report_Options) -> (begin: s64, end: s64) {
    begin: s64;
    end := trace.nodes.count;

    while end > 0 {
        info := trace.nodes[end - 1].info;
        if info && !procedure_matches(info, options.roots_to_skip) break;
        end -= 1;
    }

    for index: 0..end - 1 if procedure_matches(trace.nodes[index].info, options.known_allocation_leaves) {
        begin = index;
        break;
    }

    return begin, end;
}

procedure_matches :: (info: *Stack_Trace_Procedure_Info, procedures: [] Procedure_To_Elide) -> bool {
    if !info || info.name.count == 0 return false;

    for procedure: procedures {
        if info.name != procedure.procedure_name continue;

        fragment := procedure.module_name;
        if fragment.count == 0 return true;

        path := info.location.fully_pathed_filename;
        if fragment.count > path.count continue;

        for start: 0..path.count - fragment.count {
            matches := true;

            for byte, index: fragment if path[start + index] != byte {
                matches = false;
                break;
            }

            if matches return true;
        }
    }

    return false;
}

trace_matches :: (trace: *Packed_Stack_Trace, procedures: [] Procedure_To_Elide) -> bool {
    if !trace return false;
    for node: trace.nodes if procedure_matches(node.info, procedures) return true;
    return false;
}

traces_equal :: (a: *Packed_Stack_Trace, b: *Packed_Stack_Trace) -> bool {
    if !a || !b return a == b;
    if a.nodes.count != b.nodes.count return false;
    for node, index: a.nodes if node.info != b.nodes[index].info || node.line_number != b.nodes[index].line_number return false;
    return true;
}

// What identifies an allocation in the report's table. A block can start where a range does, so it is told apart by
// its record.
report_key :: (a: *Allocation) -> *void {
    return ifx a.block then cast(*void) a else a.memory;
}

// A sort key with the position of the thing it came from.
Sort_Item :: struct {
    key: u64;
    position: s64;
}

// Allocations ordered by start address. `reach[i]` is the greatest end among the first `i + 1` items, so a search can
// stop as soon as nothing earlier still covers the address.
Address_Set :: struct {
    items: [] Sort_Item;
    reach: [] u64;
}

// Lookup by address over the report's allocations, built once per report. Ranges and blocks are kept apart because a
// block covers the ranges inside it.
Address_Index :: struct {
    work: [] Report_Allocation;
    ranges: Address_Set;
    blocks: Address_Set;
}

// Stable bottom-up merge sort by key. `scratch` must be as long as `items`.
sort_items :: (items: [] Sort_Item, scratch: [] Sort_Item) {
    source := items;
    target := scratch;
    width := 1;

    while width < items.count {
        low := 0;

        while low < items.count {
            middle := min(low + width, items.count);
            high := min(low + 2 * width, items.count);
            left := low;
            right := middle;
            out := low;

            while left < middle && right < high {
                if source[right].key < source[left].key {
                    target[out] = source[right];
                    right += 1;
                } else {
                    target[out] = source[left];
                    left += 1;
                }

                out += 1;
            }

            while left < middle {
                target[out] = source[left];
                left += 1;
                out += 1;
            }

            while right < high {
                target[out] = source[right];
                right += 1;
                out += 1;
            }

            low += 2 * width;
        }

        swapped := source;

        source = target;
        target = swapped;
        width *= 2;
    }

    if source.data != items.data for item, i: source items[i] = item;
}

sort_by_ordinal :: (work: [] Report_Allocation, pool: *RP.Pool) {
    items := ([] Sort_Item).{ work.count, cast(*Sort_Item) pool_memory(pool, work.count * size_of(Sort_Item)) };
    spare := ([] Sort_Item).{ work.count, cast(*Sort_Item) pool_memory(pool, work.count * size_of(Sort_Item)) };
    copy := ([] Report_Allocation).{ work.count, cast(*Report_Allocation) pool_memory(pool, work.count * size_of(Report_Allocation)) };

    for item, i: work {
        items[i] = .{ cast(u64) item.allocation.ordinal, i };
        copy[i] = item;
    }

    sort_items(items, spare);
    for item, i: items work[i] = copy[item.position];
}

build_address_set :: (work: [] Report_Allocation, blocks: bool, pool: *RP.Pool) -> Address_Set {
    count: s64;
    for item: work if item.allocation.block == blocks count += 1;

    set: Address_Set;
    set.items = .{ count, cast(*Sort_Item) pool_memory(pool, count * size_of(Sort_Item)) };
    set.reach = .{ count, cast(*u64) pool_memory(pool, count * size_of(u64)) };
    spare := ([] Sort_Item).{ count, cast(*Sort_Item) pool_memory(pool, count * size_of(Sort_Item)) };
    cursor: s64;

    for item, position: work if item.allocation.block == blocks {
        set.items[cursor] = .{ cast(u64) item.allocation.memory, position };
        cursor += 1;
    }

    sort_items(set.items, spare);

    furthest: u64;

    for item, i: set.items {
        furthest = max(furthest, item.key + cast(u64) work[item.position].allocation.size);
        set.reach[i] = furthest;
    }

    return set;
}

build_address_index :: (work: [] Report_Allocation, pool: *RP.Pool) -> Address_Index {
    index: Address_Index;

    index.work = work;
    index.ranges = build_address_set(work, false, pool);
    index.blocks = build_address_set(work, true, pool);

    return index;
}

// How many items start at or below the address.
upper_bound :: (set: *Address_Set, address: u64) -> s64 {
    low := 0;
    high := set.items.count;

    while low < high {
        middle := low + (high - low) / 2;
        if set.items[middle].key <= address low = middle + 1;
        else high = middle;
    }

    return low;
}

// The earliest report position in the set whose allocation holds the address, or -1.
first_containing :: (index: *Address_Index, set: *Address_Set, address: u64) -> s64 {
    best := -1;
    i := upper_bound(set, address) - 1;

    while i >= 0 && set.reach[i] > address {
        item := set.items[i];
        inside := address - item.key < cast(u64) index.work[item.position].allocation.size;
        if inside && (best < 0 || item.position < best) best = item.position;
        i -= 1;
    }

    return best;
}

mark_containing :: (index: *Address_Index, set: *Address_Set, address: u64, root: bool) {
    i := upper_bound(set, address) - 1;

    while i >= 0 && set.reach[i] > address {
        item := set.items[i];

        if address - item.key < cast(u64) index.work[item.position].allocation.size {
            target := *index.work[item.position];
            target.entry.reachable = true;
            if root target.entry.exempt = true;
        }

        i -= 1;
    }
}

containing_allocation :: (pointer: *void, index: *Address_Index) -> *Report_Allocation {
    if !pointer return null;

    address := cast(u64) pointer;

    // A range lies inside the block it came from, so the range is the closer owner.
    position := first_containing(index, *index.ranges, address);

    if position < 0 position = first_containing(index, *index.blocks, address);
    if position < 0 return null;

    return *index.work[position];
}

mark_range :: (data: *u8, size: s64, index: *Address_Index, root := false) {
    for offset: 0..size - size_of(*void) {
        pointer: *void;

        memcpy(*pointer, data + offset, size_of(*void));

        // A pointer reaches the block it lies in and the range it lies in.
        if pointer {
            address := cast(u64) pointer;

            mark_containing(index, *index.ranges, address, root);
            mark_containing(index, *index.blocks, address, root);
        }
    }
}

build_groups :: (report: *Leak_Report, groups: **Group_Info, count: s64) {
    for index: 0..count - 1 {
        group := groups[index];
        if !group continue;

        members: s64;
        for summary: report.sorted_summaries if summary.group_info == group members += 1;

        group.sub_summaries = .{ members, cast(**Leak_Summary) report_memory(report, members * size_of(*Leak_Summary)) };
        cursor: s64;

        for summary: report.sorted_summaries if summary.group_info == group {
            group.sub_summaries[cursor] = summary;
            cursor += 1;
        }

        first := group.sub_summaries[0].alloc_site_trace;
        if !first continue;

        roots := first.nodes.count;
        leaves := roots;

        for summary: group.sub_summaries {
            trace := summary.alloc_site_trace;
            if !trace {
                roots = 0;
                leaves = 0;
                break;
            }

            roots = min(roots, trace.nodes.count);
            leaves = min(leaves, trace.nodes.count);
            common: s64;
            while common < roots && first.nodes[first.nodes.count - 1 - common].info == trace.nodes[trace.nodes.count - 1 - common].info common += 1;

            roots = common;
            common = 0;
            while common < leaves && first.nodes[common].info == trace.nodes[common].info common += 1;

            leaves = common;
        }

        group.root_nodes_in_common = roots;
        group.leaf_nodes_in_common = leaves;
    }
}
