Procedure_To_Elide :: struct {
    procedure_name: string;
    module_name: string;
}

Group_Info :: struct {
    procedure: Procedure_To_Elide;
    sub_summaries: [] *Leak_Summary;
    root_nodes_in_common: s64;
    leaf_nodes_in_common: s64;
}

Leak_Summary :: struct {
    count: s64;
    bytes: s64;
    allocations_since_last_visualizer_update: s64;
    bytes_allocated_since_last_visualizer_update: s64;
    alloc_site_trace: *Packed_Stack_Trace;
    allocator_proc: Allocator_Proc;
    group_info: *Group_Info;
    child_count: s64;
    child_bytes: s64;
}

Leak_Report_Options :: struct {
    known_allocation_leaves: [] Procedure_To_Elide;
    roots_to_skip: [] Procedure_To_Elide;
    group_by_these_calls: [] Procedure_To_Elide;
    scan_global_memory_to_remove_leaks := true;
    scan_leaks_to_combine := true;
}

Leak_Report :: struct {
    marked_as_non_leak_allocations: s64;
    marked_as_non_leak_bytes: s64;
    sorted_summaries: [] *Leak_Summary;
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
    reachable: bool;
}

#scope_file
Leak_Table :: #import "Hash_Table";
Leak_Pool :: #import "Pool";
