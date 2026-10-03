# Protocol_For_Memory_Visualization


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `Vis_Protocol_Tag :: enum u32`
- `Memory_Debugger_Version_Info :: struct`
- `Single_Stack_Trace_Info :: struct`
- `Packed_Packed_Stack_Trace :: struct`
- `Summary_Info :: struct`
- `Summary_Common :: struct`
- `Packed_Summary :: struct`
- `Allocs_This_Frame :: struct`
- `Frees_This_Frame :: struct`
- `String_Index :: #type,isa s32`
- `String_Info :: struct`
- `Packed_Stack_Trace_Procedure_Info :: struct`
- `Alloc_And_Free_Traces :: struct`
- `compare_alloc_and_free_traces :: (a: Alloc_And_Free_Traces, b: Alloc_And_Free_Traces) -> bool`
- `MEMORY_VISUALIZATION_CHANNEL_NAME :: "Jai Memory Visualization"`
- `VIS_PROTOCOL_VERSION :: 2`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Alloc_And_Free_Traces`

```jai
Alloc_And_Free_Traces :: struct {
    alloc_site_trace: *void;
    free\_site_trace: *void;
}
```
### `Allocs_This_Frame`

```jai
Allocs_This_Frame :: struct {
    using #as common: Summary_Common;
}
```
### `Frees_This_Frame`

```jai
Frees_This_Frame :: struct {
    using #as common: Summary_Common;

    free\_site_trace: *void;
}
```
### `Memory_Debugger_Version_Info`

```jai
Memory_Debugger_Version_Info :: struct {
    tag:       Vis_Protocol_Tag;
    _unused:   u32;
    version:   u64;
    default_allocator_proc: Allocator_Proc;
}
```
### `Packed_Packed_Stack_Trace`

```jai
Packed_Packed_Stack_Trace :: struct {
    info: Single_Stack_Trace_Info;
    nodes: [] Stack_Trace_Node;
}
```
### `Packed_Stack_Trace_Procedure_Info`

```jai
Packed_Stack_Trace_Procedure_Info :: struct {
    tag:              Vis_Protocol_Tag;
    name:             String_Index;

    filename:         String_Index;
    line_number:      s32;
    character_number: s32;

    remote_address:   *void;
}
```
### `Packed_Summary`

```jai
Packed_Summary :: struct {
    using #as common: Summary_Common;

    allocations_since_last_visualizer_update: s64;
    bytes_allocated_since_last_visualizer_update: s64;
    grouped_from_this_many_stack_traces: s32;
    root_nodes_in_common: s32;
    leaf_nodes_in_common: s32;
}
```
### `Single_Stack_Trace_Info`

```jai
Single_Stack_Trace_Info :: struct {
    tag:       Vis_Protocol_Tag;
    num_nodes: s32;
    hash:      u64;
    address:   *void;
}
```
### `String_Info`

```jai
String_Info :: struct {
    tag:       Vis_Protocol_Tag;
    index:     String_Index;
    count:     s32;
}
```
### `Summary_Common`

```jai
Summary_Common :: struct {
    count: s64;
    bytes: s64;
    alloc_site_trace: *void;
    allocator_proc: Allocator_Proc;

}
```
### `Summary_Info`

```jai
Summary_Info :: struct {
    tag:                   Vis_Protocol_Tag;

    num_live:              s32;
    num_allocs_this_frame: s32;
    num_frees\_this_frame: s32;
}
```
### `Vis_Protocol_Tag`

```jai
Vis_Protocol_Tag :: enum u32 {
    ERROR                  :: 0;
    RESET_AND_TELL_VERSION :: 1;
    ADD_STACK_TRACE        :: 2;
    ADD_PROCEDURE_INFO     :: 3;
    ADD_STRING             :: 4;
    UPDATE_USAGE           :: 5;
}
```
## Contracts and constraints

This page defines the version-2 packet tags and record layouts exchanged with the memory visualizer. Messages carry stack-trace/procedure/string identifiers and allocator usage summaries; tag-specific fields determine the following payload, and the receiver must process the reset/version handshake before interpreting later messages. The declarations are a wire protocol, so field widths/order and pointer-sized fields must match both endpoints; `remote_address` and trace pointers are identifiers, not dereferenceable cross-process pointers. The channel name and version are compatibility keys.
