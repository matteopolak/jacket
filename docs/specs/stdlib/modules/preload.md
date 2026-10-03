# Preload


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `Operating_System_Tag :: enum u32`
- `CPU_Tag :: enum u32`
- `Type_Info_Tag :: enum u32`
- `Type_Info :: struct`
- `Type_Info_Integer :: struct`
- `Type_Info_Float :: struct`
- `Type_Info_String :: struct`
- `Type_Info_Pointer :: struct`
- `Type_Info_Procedure :: struct`
- `Struct_Nontextual_Flags :: enum_flags u32`
- `Struct_Textual_Flags :: enum_flags u32`
- `Struct_Status_Flags :: enum_flags u32`
- `Type_Info_Struct :: struct`
- `Type_Info_Tagged_Union_Binding :: struct`
- `Type_Info_Struct_Member :: struct`
- `Type_Info_Array :: struct`
- `Enum_Status_Flags :: enum_flags u16`
- `Enum_Type_Flags :: enum_flags u16`
- `Type_Info_Enum :: struct`
- `Type_Info_Variant_Flags :: enum_flags u32`
- `Type_Info_Variant :: struct`
- `Any_Struct :: struct`
- `__reg :: #type,distinct u16`
- `Workspace :: s64`
- `Allocator_Proc :: #type (mode: Allocator_Mode, size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void`
- `Allocator :: struct`
- `Allocator_Mode :: enum`
- `Allocator_Caps :: enum_flags u32`
- `Logger :: #type (message: string, data: *void, info: Log_Info)`
- `Log_Info :: struct`
- `Log_Level :: enum u8`
- `Log_Flags :: enum_flags u32`
- `Log_Section :: struct`
- `Stack_Trace_Procedure_Info :: struct`
- `Stack_Trace_Node :: struct`
- `__command_line_arguments: [] *u8`
- `__runtime_support_disable_stack_trace := false`
- `Source_Code_Location :: struct`
- `Source_Code_Range :: struct`
- `get_current_workspace :: () -> Workspace #compiler`
- `memcpy :: (dest: *void, source: *void, count: s64) #intrinsic`
- `memcmp :: (a: *void, b: *void, count: s64) -> s16 #intrinsic`
- `memset :: (dest: *void, value: u8, count: s64) #intrinsic`
- `compare_and_swap :: (pointer: *$T, old: T, new: T) -> (success: bool, old_value: T) #intrinsic`
- `atomic_exchange :: (pointer: *$T, new: T) -> old_value: T #intrinsic`
- `atomic_load :: (pointer: *$T) -> T #intrinsic`
- `atomic_store :: (pointer: *$T, new: T) #intrinsic`
- `For_Flags :: enum_flags u8`
- `Procedure_With_Data :: struct`
- `Newstring :: struct`
- `Array_View_64 :: struct`
- `Resizable_Array :: struct`
- `Name_Mapper :: #type ([] string)`
- `FIRST_ADD_CONTEXT :: #code #add_context #as using base: Context_Base`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Allocator`

```jai
Allocator :: struct {
    proc: Allocator_Proc;
    data: *void;
}
```
### `Allocator_Caps`

```jai
Allocator_Caps :: enum_flags u32 {
    MULTIPLE_THREADS :: 0x1;
    CREATE_HEAP      :: 0x2;
    FREE             :: 0x4;
    ACTUALLY_RESIZE  :: 0x8;
    IS_THIS_YOURS    :: 0x10;

    HINT_I_AM_A_FAST_BUMP_ALLOCATOR       :: 0x0100_0000;
    HINT_I_AM_A_GENERAL_HEAP_ALLOCATOR    :: 0x0200_0000;
    HINT_I_AM_PER_FRAME_TEMPORARY_STORAGE :: 0x0400_0000;
    HINT_I_AM_A_DEBUG_ALLOCATOR           :: 0x0800_0000;
}
```
### `Allocator_Mode`

```jai
Allocator_Mode :: enum {
    ALLOCATE  :: 0;
    RESIZE    :: 1;
    FREE      :: 2;
    STARTUP   :: 3;
    SHUTDOWN  :: 4;
    THREAD_START  :: 5;
    THREAD_STOP   :: 6;
    CREATE_HEAP   :: 7;
    DESTROY_HEAP  :: 8;
    IS_THIS_YOURS :: 9;
    CAPS          :: 10;
}
```
For mode `CAPS`, the allocator procedure returns capability flags encoded in its `*void` result. In Basic's `get_capabilities` call, `old_memory` points to a caller-provided `string` slot for the allocator name; size arguments are zero and `allocator_data` is null. The allocator supplies the name storage and lifetime; Basic does not copy it.
### `Any_Struct`

```jai
Any_Struct :: struct {
    type: *Type_Info;
    value_pointer: *void;
}
```
### `Array_View_64`

```jai
Array_View_64 :: struct {
    count     : s64;
    data      : *u8;
}
```
### `CPU_Tag`

```jai
CPU_Tag :: enum u32 {
    UNINITIALIZED :: 0;
    KRAMPU        :: 1;
    CUSTOM        :: 2;
    X64           :: 3;
    ARM64         :: 4;
}
```
### `Enum_Status_Flags`

```jai
Enum_Status_Flags :: enum_flags u16 {
    INCOMPLETE :: 0x1;
}
```
### `Enum_Type_Flags`

```jai
Enum_Type_Flags :: enum_flags u16 {
    FLAGS     :: 0x1;
    COMPLETE  :: 0x2;
    SPECIFIED :: 0x4;
}
```
### `For_Flags`

```jai
For_Flags :: enum_flags u8 {
    POINTER :: 0x1;
    REVERSE :: 0x2;
}
```
### `Log_Flags`

```jai
Log_Flags :: enum_flags u32 {
    NONE              :: 0x0;
    ERROR             :: 0x1;
    WARNING           :: 0x2;
    CONTENT           :: 0x4;
    TO_FILE_ONLY      :: 0x8;
    VERBOSE_ONLY      :: 0x10;
    VERY_VERBOSE_ONLY :: 0x20;
    TOPIC_ONLY        :: 0x40;
}
```
### `Log_Info`

```jai
Log_Info :: struct {
    source_identifier:   u64;
    location:     Source_Code_Location;
    common_flags: Log_Flags;
    user_flags:   u32;
    section:      *Log_Section;
}
```
### `Log_Level`

```jai
Log_Level :: enum u8 {
    NORMAL       :: 0;
    VERBOSE      :: 1;
    VERY_VERBOSE :: 2;
}
```
### `Log_Section`

```jai
Log_Section :: struct {
    name:       string;
}
```
### `Newstring`

```jai
Newstring    :: struct {
    count     : s64;
    data      : *u8;
}
```
### `Operating_System_Tag`

```jai
Operating_System_Tag :: enum u32 {
    NONE       :: 0;
    KRAMPOS    :: 1;
    WINDOWS    :: 2;
    LINUX      :: 3;
    ANDROID    :: 4;
    IOS        :: 5;
    MACOS      :: 6;
    NN_SWITCH  :: 7;
    PS4        :: 8;
    PS5        :: 9;
    XBOX       :: 10;
    WASM       :: 11;
    NN_SWITCH2 :: 12;
}
```
### `Procedure_With_Data`

```jai
Procedure_With_Data :: struct {
    proc      : *void;
    data      : *void;
}
```
### `Resizable_Array`

```jai
Resizable_Array :: struct {
    count      : s64;
    data       : *void;
    allocated  : s64;

    allocator  : Allocator;
}
```
### `Source_Code_Location`

```jai
Source_Code_Location :: struct {
    fully_pathed_filename:  string;
    line_number:            s64;
    character_number:       s64;
}
```
### `Source_Code_Range`

```jai
Source_Code_Range :: struct {
    fully_pathed_filename:  string;
    line_number_start:      s64;
    line_number_end:        s64;
    character_number_start: s64;
    character_number_end:   s64;
}
```
### `Stack_Trace_Node`

```jai
Stack_Trace_Node :: struct {
    next:        *Stack_Trace_Node;
    info:        *Stack_Trace_Procedure_Info;
    hash:        u64;
    call_depth:  u32;
    line_number: u32;
}
```
### `Stack_Trace_Procedure_Info`

```jai
Stack_Trace_Procedure_Info :: struct {
    name:      string;
    location:  Source_Code_Location;

    procedure_address: *void;

}
```
### `Struct_Nontextual_Flags`

```jai
Struct_Nontextual_Flags :: enum_flags u32 {
    NOT_INSTANTIABLE          :: 0x4;
    ALL_MEMBERS_UNINITIALIZED :: 0x40;
    POLYMORPHIC               :: 0x100;
}
```
### `Struct_Status_Flags`

```jai
Struct_Status_Flags :: enum_flags u32 {
    INCOMPLETE :: 0x1;
    LOCAL      :: 0x4;
}
```
### `Struct_Textual_Flags`

```jai
Struct_Textual_Flags :: enum_flags u32 {
    FOREIGN                                :: 0x1;
    UNION                                  :: 0x2;
    NO_PADDING                             :: 0x4;
    TYPE_INFO_NONE                         :: 0x8;
    TYPE_INFO_NO_SIZE_COMPLAINT            :: 0x10;
    TYPE_INFO_PROCEDURES_ARE_VOID_POINTERS :: 0x20;
    UNION_IS_TAGGED                        :: 0x40;
}
```
### `Type_Info`

```jai
Type_Info :: struct {
    type: Type_Info_Tag;
    runtime_size:    s64;
}
```
### `Type_Info_Array`

```jai
Type_Info_Array :: struct {
    using #as info: Type_Info;

    element_type: *Type_Info;

    array_type:            Array_Type;

    array_count:           s64;

    Array_Type :: enum u16 {
        FIXED     :: 0;
        VIEW      :: 1;
        RESIZABLE :: 2;
    }
}
```
### `Type_Info_Enum`

```jai
Type_Info_Enum :: struct {
    using #as info: Type_Info;

    name: string;
    internal_type: *Type_Info_Integer;

    names:         [] string;
    values:        [] s64;

    status_flags:    Enum_Status_Flags;
    enum_type_flags: Enum_Type_Flags;
}
```
### `Type_Info_Float`

```jai
Type_Info_Float :: struct {
    using #as info: Type_Info;
}
```
### `Type_Info_Integer`

```jai
Type_Info_Integer :: struct {
    using #as info: Type_Info;
    signed: bool;
}
```
### `Type_Info_Pointer`

```jai
Type_Info_Pointer :: struct {
    using #as info: Type_Info;

    pointer_to: *Type_Info;
}
```
### `Type_Info_Procedure`

```jai
Type_Info_Procedure :: struct {
    using #as info: Type_Info;

    argument_types: [] *Type_Info;
    return_types:   [] *Type_Info;

    procedure_flags: Flags;

    Flags :: enum_flags u32 {
        IS_ELSEWHERE    :: 0x1;
        IS_COMPILE_TIME_ONLY :: 0x2;
        IS_POLYMORPHIC  :: 0x4;
        HAS_NO_CONTEXT  :: 0x8;
        IS_C_CALL       :: 0x20;
        IS_INTRINSIC    :: 0x80;
        IS_SYMMETRIC    :: 0x100;

        IS_CPP_METHOD   :: 0x1000_0000;
        HAS_CPP_NON_POD_RETURN_TYPE :: 0x2000_0000;
    }
}
```
### `Type_Info_String`

```jai
Type_Info_String :: struct {
    using #as info: Type_Info;
}
```
### `Type_Info_Struct`

```jai
Type_Info_Struct :: struct {
    using #as info: Type_Info;

    name: string;

    parameters:           [] Type_Info_Struct_Member;

    specified_parameters: [] Type_Info_Struct_Member;
    members:              [] Type_Info_Struct_Member;

    tagged_union_bindings: [] Type_Info_Tagged_Union_Binding;

    status_flags:     Struct_Status_Flags;
    nontextual_flags: Struct_Nontextual_Flags;
    textual_flags:    Struct_Textual_Flags;

    alignment: s32;

    polymorph_source_struct: *Type_Info_Struct;

    initializer: (*void) #no_context;

    constant_storage: [] u8;
    notes: [] string;
}
```
### `Type_Info_Struct_Member`

```jai
Type_Info_Struct_Member :: struct {
    name: string;
    type: *Type_Info;
    offset_in_bytes: s64;

    flags: Flags;

    notes: [] string;

    offset_into_constant_storage: s64 = -1;

    Flags :: enum_flags u32 {
        CONSTANT                              :: 0x1;
        IMPORTED                              :: 0x2;
        USING                                 :: 0x4;
        PROCEDURE_WITH_VOID_POINTER_TYPE_INFO :: 0x8;
        AS                                    :: 0x10;
        OVERLAY                               :: 0x20;
    }
}
```

`members` is the reflected member sequence and `Type_Info_Struct_Member.flags`
identifies metadata such as constant, imported, `using`, `as`, overlay, and
procedure-with-void-pointer-type-info members. A public reflection example
iterates this sequence and includes procedure members; the
`TYPE_INFO_PROCEDURES_ARE_VOID_POINTERS` option changes the recorded type for
procedure members rather than establishing that they disappear. Negative
`offset_in_bytes` is used for constant members. Direct-declaration cases show that procedure
constants and nested type constants are included in declaration order: a record
declaring procedure `f`, field `n`, nested type `Item`, then field `tail` reports
those four members in that order, with negative offsets for the constants. The
default `Hash_Table.Table(string, int)` sequence is `count`, `allocated`,
`slots_filled`, `allocator`, `Entry`, `entries`, `hash_function`,
`compare_function`, `SIZE_MIN`; `entries` is therefore index 5 for that exact
canonical declaration. This does not specify all inherited, imported, or
conditional-member ordering variants. Names and flags identify members across
other shapes; a fixed index is not a universal layout rule.

### `Type_Info_Tag`

```jai
Type_Info_Tag :: enum u32 {
    INTEGER              :: 0;
    FLOAT                :: 1;
    BOOL                 :: 2;
    STRING               :: 3;
    POINTER              :: 4;
    PROCEDURE            :: 5;
    VOID                 :: 6;
    STRUCT               :: 7;
    ARRAY                :: 8;
    OVERLOAD_SET         :: 9;
    ANY                  :: 10;
    ENUM                 :: 11;
    POLYMORPHIC_VARIABLE :: 12;
    TYPE                 :: 13;
    CODE                 :: 14;
    UNTYPED_LITERAL      :: 15;
    UNTYPED_ENUM         :: 16;

    VARIANT              :: 18;
}
```
### `Type_Info_Tagged_Union_Binding`

```jai
Type_Info_Tagged_Union_Binding :: struct {
    constant_value: u64;
    member_index: s32;
}
```
### `Type_Info_Variant`

```jai
Type_Info_Variant :: struct {
    using #as info: Type_Info;

    name:          string;
    variant_of:    *Type_Info;
    variant_flags: Type_Info_Variant_Flags;
}
```
### `Type_Info_Variant_Flags`

```jai
Type_Info_Variant_Flags :: enum_flags u32 {
    DISTINCT :: 0x1;
    ISA      :: 0x2;
}
```
## Contracts and constraints

This file defines compiler-preloaded core type metadata and target/CPU tags needed before ordinary modules can be compiled. It intentionally stays declarative and dependency-light: metadata layouts can be incomplete during compilation (for example, `runtime_size` may be `-1` for incomplete or polymorphic types), and several tags have no separate specialized metadata struct. Changes here affect compiler bootstrap and every later type-info consumer, so field order, flags, and representations are ABI-like compiler contracts.
