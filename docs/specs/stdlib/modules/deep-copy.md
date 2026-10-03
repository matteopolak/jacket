# Deep_Copy

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `Deep_Copy_Config :: struct`
- `Deep_Copy :: (value: *$T, config: Deep_Copy_Config, preallocated_root: *void = null, pointer_map: *Pointer_Map = null, loc := #caller_location) -> *T`
- `Pointer_Map :: struct`

- `helper_count := 0`
- `time_total: float64`
- [DC_DEBUG] `indent := 0`

## Public type members

#### `Deep_Copy_Config` (struct)

- `string_mapper_proc: (input: string, data: *void) -> (output: string)`
- `string_mapper_data: *void`
- `follow_struct_pointers := true`

#### `Pointer_Map` (struct)

- `table: Table(*void, *void)`

## Behavioral contract and constraints

`Deep_Copy` copies a pointed-to object graph, with an optional preallocated root, pointer map, string-mapping callback, and pointer-following policy. It returns null for a null input. The module explicitly warns that fixed arrays can be mishandled, `Any` is not copied, and pointers into the middle of a shared allocation may not be recognized correctly. Treat those shapes as unsupported. Allocations use the active allocator; callbacks and pointer maps are caller-owned inputs.


## Module parameters

```jai
#module_parameters(DC_DEBUG := false, DEBUG_PRINT := false);
```
`indent` is exported only when `DC_DEBUG` is true. `DEBUG_PRINT` is ignored while `DC_DEBUG` is false.
