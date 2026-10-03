VIS_PROTOCOL_VERSION :: 2;
MEMORY_VISUALIZATION_CHANNEL_NAME :: "Jai Memory Visualization";

Vis_Protocol_Tag :: enum u32 {
    ERROR :: 0;
    RESET_AND_TELL_VERSION :: 1;
    ADD_STACK_TRACE :: 2;
    ADD_PROCEDURE_INFO :: 3;
    ADD_STRING :: 4;
    UPDATE_USAGE :: 5;
}

Memory_Debugger_Version_Info :: struct {
    tag: Vis_Protocol_Tag;
    _unused: u32;
    version: u64;
    default_allocator_proc: Allocator_Proc;
}

String_Index :: #type,isa s32;

String_Info :: struct {
    tag: Vis_Protocol_Tag;
    index: String_Index;
    count: s32;
}

Packed_Stack_Trace_Procedure_Info :: struct {
    tag: Vis_Protocol_Tag;
    name: String_Index;
    filename: String_Index;
    line_number: s32;
    character_number: s32;
    remote_address: *void;
}

Single_Stack_Trace_Info :: struct {
    tag: Vis_Protocol_Tag;
    num_nodes: s32;
    hash: u64;
    address: *void;
}

Packed_Packed_Stack_Trace :: struct {
    info: Single_Stack_Trace_Info;
    nodes: [] Stack_Trace_Node;
}

Summary_Info :: struct {
    tag: Vis_Protocol_Tag;
    num_live: s32;
    num_allocs_this_frame: s32;
    num_frees_this_frame: s32;
}

Summary_Common :: struct {
    count: s64;
    bytes: s64;
    alloc_site_trace: *void;
    allocator_proc: Allocator_Proc;
}

Packed_Summary :: struct {
    using #as common: Summary_Common;
    allocations_since_last_visualizer_update: s64;
    bytes_allocated_since_last_visualizer_update: s64;
    grouped_from_this_many_stack_traces: s32;
    root_nodes_in_common: s32;
    leaf_nodes_in_common: s32;
}

Allocs_This_Frame :: struct {
    using #as common: Summary_Common;
}

Frees_This_Frame :: struct {
    using #as common: Summary_Common;
    free_site_trace: *void;
}

Alloc_And_Free_Traces :: struct {
    alloc_site_trace: *void;
    free_site_trace: *void;
}

compare_alloc_and_free_traces :: (a: Alloc_And_Free_Traces, b: Alloc_And_Free_Traces) -> bool {
    return a.alloc_site_trace == b.alloc_site_trace && a.free_site_trace == b.free_site_trace;
}
