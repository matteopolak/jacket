// Names a procedure in a stack trace, for the lists in [Leak_Report_Options].
Procedure_To_Elide :: struct {
    procedure_name: string; // the procedure's name, matched exactly
    module_name: string; // a part of its file's path, or empty to match any file
}

// The leaks whose trace passes through one of [Leak_Report_Options.group_by_these_calls].
Group_Info :: struct {
    procedure: Procedure_To_Elide;
    sub_summaries: [] *Leak_Summary;
    root_nodes_in_common: s64; // outermost frames every trace in the group shares
    leaf_nodes_in_common: s64; // innermost frames every trace in the group shares
}

// The leaked allocations made from one place: the same stack trace, after [Leak_Report_Options]
// trims it.
Leak_Summary :: struct {
    count: s64; // leaked allocations
    bytes: s64; // their total size
    allocations_since_last_visualizer_update: s64;
    bytes_allocated_since_last_visualizer_update: s64;
    alloc_site_trace: *Packed_Stack_Trace;
    allocator_proc: Allocator_Proc; // the allocator they came from; null when they came from several
    group_info: *Group_Info;
    child_count: s64; // other leaks reachable only through these, which are not listed on their own
    child_bytes: s64;
}

// What [make_leak_report] and [report_memory_leaks] count and how they print traces.
Leak_Report_Options :: struct {
    // A trace is printed from the first of these frames outward, which hides your allocation wrappers.
    known_allocation_leaves: [] Procedure_To_Elide;
    // Outermost frames to leave out of printed traces, such as `main` or a thread's entry.
    roots_to_skip: [] Procedure_To_Elide;
    // Leaks whose trace passes through one of these are grouped under it.
    group_by_these_calls: [] Procedure_To_Elide;
    // An allocation that a pointer in global data or in a reachable allocation points to is not a leak.
    scan_global_memory_to_remove_leaks := true;
    // A leak that only another leak points to is counted as that one's child instead of on its own.
    scan_leaks_to_combine := true;
}

// The leaks the memory debugger found, from [make_leak_report]. Free it with [deinit].
//
// With Basic's `MEMORY_DEBUGGER` parameter off nothing is tracked and the report is empty.
Leak_Report :: struct {
    // Allocations marked with [this_allocation_is_not_a_leak] or made before
    // [allocations_before_this_call_will_not_be_considered_leaks].
    marked_as_non_leak_allocations: s64;
    marked_as_non_leak_bytes: s64;
    sorted_summaries: [] *Leak_Summary; // most bytes first, children included
    leak_combiner_table: Leak_Table.Table(*void, Leak_Combiner_Record);
    pool: Leak_Pool.Pool;
}

#scope_module
Packed_Stack_Trace :: struct {
    nodes: [] Stack_Trace_Node;
}

Leak_Combiner_Record :: struct {
    summary: *Leak_Summary;
    owner: *void;
    // The record of `owner`, or a link further up its chain once the report has shortened the path.
    parent: *Leak_Combiner_Record;
    reachable: bool;
    exempt: bool;
}

#scope_file
Leak_Table :: #import "Hash_Table";
Leak_Pool :: #import "Pool";
