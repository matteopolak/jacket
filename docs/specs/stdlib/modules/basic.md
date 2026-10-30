# Basic

## Module parameters and configuration

The public declaration uses an empty first parameter list and puts these five settings in the second list:

```jai
#module_parameters () (MEMORY_DEBUGGER := false, ENABLE_ASSERT := true, REPLACEMENT_INTERFACE: $I/interface Memory_Debugger_Interface = Memory_Debugger_Interface, VISUALIZE_MEMORY_DEBUGGER := true, TEMP_ALLOCATOR_OVERWRITE_MEMORY := false) {
```

`VISUALIZE_MEMORY_DEBUGGER` and `REPLACEMENT_INTERFACE` affect behavior only when `MEMORY_DEBUGGER` is enabled. `TEMP_ALLOCATOR_OVERWRITE_MEMORY` enables overwrite stamping for freed temporary storage.

`Memory_Debugger_Interface` is the contextual interface type defined inside Basic's module-parameter declaration. That declaration uses it to constrain `REPLACEMENT_INTERFACE` and as the sentinel default; its four inline no-op members are `check_alloc(allocator: Allocator, memory: *void, size: s64)`, `check_free(allocator: Allocator, memory: *void)`, `check_create_heap(a: Allocator)`, and `check_destroy_heap(a: Allocator)`. This is a parameter-contract type, not a separately declared runtime service or ordinary exported value. A custom interface supplies those same hooks. When `MEMORY_DEBUGGER` is true, the sentinel selects Basic's own checker implementation; a non-sentinel replacement selects that type's hooks. The debugger configuration has no effect when `MEMORY_DEBUGGER` is false.

When memory debugging is enabled, `allocations_before_this_call_will_not_be_considered_leaks()` sets the allocation-index baseline for later leak reports. `this_allocation_is_not_a_leak(memory)` marks a tracked allocation as exempt from leak reporting; null is ignored, while an unknown non-null pointer is reported as misuse and terminates the process. `this_allocation_leaks_todo(memory)` currently has the same effect as `this_allocation_is_not_a_leak`. `this_allocation_does_not_propagate_leak_ownership(memory)` prevents the allocation from claiming the ownership of allocations reachable through it; null and unknown pointers are ignored. With memory debugging disabled, the three pointer macros compile to no-ops. `report_memory_leaks` builds and logs a report; `make_leak_report` returns an owned report that must be released with `deinit`.

`ENABLE_ASSERT` is a compile-time layout setting as well as a check setting. When true, each `String_Builder.Buffer` contains `ensured_count: s64`; builder operations use it to validate that writes fit space previously ensured. When false, that field and its checks are omitted. `STRING_BUILDER_BUFFER_SIZE` is defined as `4096 - size_of(String_Builder.Buffer)`, and the builder's inline byte storage has that length. Therefore changing `ENABLE_ASSERT` changes the public builder layout and inline capacity; code exchanging `String_Builder` values must use a consistent `Basic` configuration.

## Generic type reflection

`print_type_to_builder(builder, info, print_parameters_matching_defaults := false)` writes a type's source-like description into the supplied builder. For a parameterized type, it includes parameter names and values; by default, parameters whose values equal their declared defaults are elided. Passing `true` includes matching-default parameters too. Parameters with no declared default are retained. `type_to_string(info)` returns this description as a temporary string, so callers must copy it before the relevant temporary-storage lifetime ends.

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `Apollo_Time :: #type,isa S128`
- `operator == :: inline (a: S128, b: S128) -> bool`
- `operator ! :: inline (a: S128) -> bool`
- `operator + :: inline (a: S128, b: S128) -> S128`
- `operator - :: inline (a: S128, b: S128) -> S128`
- `operator - :: inline (a: S128) -> S128`
- `operator << :: (a: S128, $$x: u8) -> S128`
- `operator >> :: (a: S128, $$x: u8) -> S128`
- `operator / :: (dividend: S128, divisor: S128) -> S128`
- `operator / :: inline (a: S128, factor: s64) -> S128`
- `operator * :: (_a: S128, _b: S128) -> S128`
- `operator * :: (a: S128, factor: s64) -> S128 #symmetric`
- `operator < :: (a: S128, b: S128) -> bool`
- `operator <= :: (a: S128, b: S128) -> bool`
- `operator > :: (a: S128, b: S128) -> bool`
- `operator >= :: (a: S128, b: S128) -> bool`
- `operator - :: (a: U128, b: U128) -> U128`
- `APOLLO_TIME_INVALID :: Apollo_Time.{0, 0x8000_0000_0000_0000}`
- `Time_Zone :: enum u32`
- `Calendar_Time :: struct`
- `ConvertToApollo :: (m: s64, FACTOR: s64) #expand`
- `seconds_to_apollo :: (m: s64) -> Apollo_Time`
- `milliseconds_to_apollo :: (m: s64) -> Apollo_Time`
- `microseconds_to_apollo :: (m: s64) -> Apollo_Time`
- `nanoseconds_to_apollo :: (m: s64) -> Apollo_Time`
- `ConvertFromApollo :: inline (t: Apollo_Time, divisor: u64, $DO_ROUNDING: bool) -> (result: s64, success: bool)`
- `to_seconds :: (t: Apollo_Time, $DO_ROUNDING := false) -> (result: s64, success: bool)`
- `to_milliseconds :: (t: Apollo_Time, $DO_ROUNDING := false) -> (result: s64, success: bool)`
- `to_microseconds :: (t: Apollo_Time, $DO_ROUNDING := false) -> (result: s64, success: bool)`
- `to_nanoseconds :: (t: Apollo_Time, $DO_ROUNDING := false) -> (result: s64, success: bool)`
- `calendar_to_string :: (ct: Calendar_Time) -> string`
- `calendar_to_iso_string :: (ct: Calendar_Time) -> string`
- `compare_apollo_times :: (a: Apollo_Time, b: Apollo_Time) -> s32`
- `to_float64_seconds :: (input: Apollo_Time) -> (result: float64, success: bool)`
- `to_apollo :: (seconds: float64) -> Apollo_Time`
- [OS == .WINDOWS || OS == .LINUX || OS == .MACOS || OS == .ANDROID] `current_time_consensus :: () -> Apollo_Time`
- `current_time_monotonic :: () -> Apollo_Time`
- `filetime_to_apollo :: (ft: FILETIME) -> Apollo_Time`
- `to_filetime :: (at: Apollo_Time) -> (ft: FILETIME, success: bool)`
- `to_calendar :: (at: Apollo_Time, time_zone := Time_Zone.UTC) -> Calendar_Time`
- `calendar_to_apollo :: (ct: Calendar_Time) -> Apollo_Time`
- `timespec_to_apollo :: (ts: timespec) -> Apollo_Time`
- `to_timespec :: (at: Apollo_Time) -> (ts: timespec, success: bool)`
- `APOLLO_TIME_FOR_JAN_1_1970 :: Apollo_Time.{low=0xdbd87f0edb7c8000, high=0x300}`
- `FEMTOSECONDS :: 1_000_000_000_000_000`
- `array_copy :: (array: [] $T) -> [] T`
- `array_copy :: (dest: *[..] $T, source: [] T)`
- `array_view :: inline (array: [] $T, index: int) -> [] T`
- `array_view :: inline (array: [] $T, index: int, count: int) -> [] T`
- `array_free :: inline (array: [] $T)`
- `array_reset :: (array: *[] $T)`
- `array_add :: (array: *[..] $T, item: T) #no_abc`
- `array_add :: (array: *[..] $T, to_append: ..T)`
- `array_add :: (array: *[..] $T) -> *T`
- `array_add_if_unique :: (array: *[..] $T, item: T, $compare: (T, T) -> bool = null) -> bool, s64`
- `array_insert_at :: (array: *[..] $T, item: T, index: int)`
- `array_unordered_remove_by_value :: inline (array: *[] $T, item: T, $stop_after_first := false, $compare: (T, T) -> bool = null) -> s64`
- `array_unordered_remove_by_index :: inline (array_pointer: *[] $T, index: s64)`
- `array_ordered_remove_by_value :: inline (array_pointer: *[] $T, item: T) -> bool`
- `array_ordered_remove_by_index :: inline (array_pointer: *[] $T, index: s64)`
- `array_find :: (array: [] $T, item: T, $compare: (T, T) -> bool = null) -> bool, s64`
- `peek :: inline (array: [] $T) -> T`
- `peek_pointer :: inline (array: [] $T) -> *T`
- `pop :: (array: *[] $T) -> T`
- `array_free :: (array: [..] $T)`
- `array_reset :: (array: *[..] $T)`
- `array_reset_keeping_memory :: inline (array: *[..] $T)`
- `array_reserve :: inline (array: *[..] $T, desired_items: s64)`
- `array_reserve :: inline (array: *Resizable_Array, desired_items: s64, element_size: s64)`
- `array_resize :: (array: *[] $T, new_count: s64, $initialize:=true)`
- `array_resize :: (array: *[..] $T, new_count: s64, $initialize:=true)`
- `maybe_grow :: inline (array: *Resizable_Array, element_size: s64)`
- `S128 :: struct`
- `U128 :: struct`
- `U128_ERROR :: U128.{0xffffffff_ffffffff, 0xffffffff_ffffffff}`
- `S128_ERROR :: S128.{0xffffffff_ffffffff, 0xffffffff_ffffffff}`
- `multiply_64bits_with_overflow :: (a: u64, b: u64) -> (high: u64, low: u64)`
- `to_s128 :: (a: $T) -> S128 #modify`
- `to_u128 :: (a: $T) -> U128 #modify`
- `logical_shift_right :: (a: S128, $$x: u8) -> S128`
- `signed_divide_with_remainder :: inline (dividend: S128, divisor: S128) -> quotient: S128, remainder: S128, overflow_or_divide_by_zero: bool`
- `unsigned_divide_with_remainder :: (u: U128, v: U128) -> quotient: U128, remainder: U128, overflow_or_divide_by_zero: bool`
- `Procedure_To_Elide :: struct`
- `Leak_Report_Options :: struct`
- `Leak_Report :: struct`
- `Leak_Summary :: struct`
- `Group_Info :: struct`
- `Preamble :: () #expand`
- `_check_alloc :: inline (allocator: Allocator, memory: *void, size: s64)`
- `_check_free :: inline (allocator: Allocator, memory: *void)`
- `do_free_bookkeeping :: (allocation: *Allocation)`
- `_check_create_heap :: (a: Allocator)`
- `_check_destroy_heap :: (a: Allocator)`
- `add_allocation_to_table :: (memory: *void, size: s64, allocator: Allocator, trace: *Stack_Trace_Node)`
- `add_allocation_this_frame :: (a: *Allocation, size: s64)`
- `add_free_this_frame :: (a: *Allocation, size: s64)`
- `allocations_before_this_call_will_not_be_considered_leaks :: ()`
- `this_allocation_is_not_a_leak :: (memory: *void)`
- `this_allocation_leaks_todo :: (memory: *void)`
- `this_allocation_does_not_propagate_leak_ownership :: (memory: *void)`
- `report_memory_leaks :: (options: Leak_Report_Options = .{})`
- `make_leak_report :: (options: Leak_Report_Options = .{}, for_visualizer := false) -> Leak_Report`
- `log_leak_report :: (report: Leak_Report, options: Leak_Report_Options = .{})`
- `deinit :: (report: *Leak_Report)`
- `Formatter :: struct`
- `FormatInt :: struct`
- `FormatFloat :: struct`
- `FormatStruct :: struct`
- `FormatArray :: struct`
- `Print_Style :: struct`
- `#add_context print_style: Print_Style`
- `formatInt :: (value : Any, base := 10, minimum_digits := 1, padding: u8 = #char "0", digits_per_comma : u16 = 0, comma_string := "") -> FormatInt`
- `formatFloat :: (value : Any, width := -1, trailing_width := -1, mode := FormatFloat.Mode.DECIMAL, zero_removal := FormatFloat.Zero_Removal.YES) -> FormatFloat`
- `formatStruct :: (value: Any, use_long_form_if_more_than_this_many_members := 5, use_newlines_if_long_form := false) -> FormatStruct`
- `get_s64_from_void_pointer :: (pointer: *void, size_in_bytes: int) -> s64`
- `get_u64_from_void_pointer :: (pointer: *void, size_in_bytes: int) -> u64`
- `print_u64_base_n :: (builder: *String_Builder, v: u64, format: FormatInt)`
- `print_float :: (builder : *String_Builder, format : FormatFloat)`
- `print_integer :: (builder : *String_Builder, format : FormatInt)`
- `print_character :: (_c: u8)`
- `print :: (format_string: string, args: .. Any, to_standard_error := false) -> bytes_printed: s64`
- `sprint :: (format_string: string, args: .. Any) -> string`
- `tprint :: (format_string: string, args: .. Any) -> string`
- `log :: (format_string: string, args: .. Any, loc := #caller_location, flags := Log_Flags.NONE, user_flags : u32 = 0, section : *Log_Section = null)`
- `log :: (section : *Log_Section, format_string: string, args: .. Any, loc := #caller_location, flags := Log_Flags.NONE, user_flags : u32 = 0)`
- `log_error :: (format_string: string, args: .. Any, loc := #caller_location, flags := Log_Flags.NONE, user_flags : u32 = 0, section : *Log_Section = null)`
- `log_error :: (section : *Log_Section, format_string: string, args: .. Any, loc := #caller_location, flags := Log_Flags.NONE, user_flags : u32 = 0)`
- `append_indentation :: (builder: *String_Builder, format: *FormatStruct)`
- `append_struct_info :: (builder: *String_Builder, info: *Type_Info_Struct, base_pointer: *void, format : *FormatStruct = null)`
- `print_type_to_builder :: (builder: *String_Builder, info: *Type_Info, print_parameters_matching_defaults := false)`
- `print_item_to_builder :: (builder: *String_Builder, item: Any, inside_struct_or_array := true, may_be_uninitialized := false)`
- `my_is_digit :: (c: s64) -> bool`
- `parse_number :: (s: string, cursor: s64) -> (end_cursor: s64, sum: s64)`
- `print_to_builder :: (builder : *String_Builder, format_string : string, args: .. Any) -> bool`
- `print :: (builder : *String_Builder, format_string : string, args: .. Any) -> bool`
- `print :: print_to_builder`
- `print_sequence_to_builder :: (builder : *String_Builder, args: .. Any)`
- `isFormatter :: (info: *Type_Info_Struct) -> *Type_Info_Struct_Member`
- `print_bytes :: (a: *void, length: int, format_string := "% ")`
- `print_array :: (builder: *String_Builder, any: Any, info_array: *Type_Info_Array, format: FormatArray, may_be_uninitialized := false)`
- `type_to_string :: (info: *Type_Info) -> string`
- `to_string :: inline (s: *u8, length: s64) -> string`
- `to_string :: (c_string: [] u8) -> string`
- `to_c_string :: (s: string) -> *u8`
- `to_upper :: (c: u8) -> u8`
- `to_lower :: (c: u8) -> u8`
- `is_alpha :: (c: u8) -> bool`
- `is_digit :: (c: u8) -> bool`
- `is_alnum :: (c: u8) -> bool`
- `is_space :: (c: u8) -> bool`
- `is_print :: (c: u8) -> bool`
- `advance :: inline (s: *string, amount: s64 = 1)`
- `advance :: inline (s: string, amount: s64 = 1) -> string`
- `temp_c_string :: (s: string) -> *u8`
- `copy_temporary_string :: inline (s: string) -> string`
- `copy_string :: (s: string) -> string`
- `to_integer :: (s: string) -> result: int, success: bool, remainder: string`
- `string_to_int :: (t: string, base := 10, $T := int) -> result: T, success: bool, remainder: string`
- `eat_spaces :: (_s: string) -> string`
- `STRING_BUILDER_BUFFER_SIZE :: 4096 - size_of(String_Builder.Buffer)`
- `String_Builder :: struct`
- `init_string_builder :: (using builder: *String_Builder, buffer_size := -1)`
- `free_buffers :: (using builder: *String_Builder)`
- `reset :: (using builder: *String_Builder)`
- `ensure_contiguous_space :: (using builder: *String_Builder, bytes: s64) -> pointer: *void`
- `ensure_contiguous_space_and_return_available_range :: (using builder: *String_Builder, minimum_bytes := 1) -> [] u8`
- `advance_through_ensured_space :: (using builder: *String_Builder, bytes: s64)`
- `append :: (using builder: *String_Builder, s: *u8, length: s64)`
- `append :: inline (builder: *String_Builder, s: string)`
- `append :: inline (builder: *String_Builder, byte: u8)`
- `append_by_pointer :: inline (builder: *String_Builder, pointer: *$T)`
- `builder_string_length :: (using builder: *String_Builder) -> int`
- `builder_to_string :: (builder: *String_Builder, extra_bytes_to_prepend := 0, do_reset := true) -> string`
- `write_builder :: (using builder: *String_Builder, do_reset := true, to_standard_error := false) -> s64`
- `consume_u8_and_length :: (input: *string, s: *u8, length: s64) -> bool`
- `get_current_buffer :: inline (using builder: *String_Builder) -> *String_Builder.Buffer`
- `get_base_buffer :: inline (using builder: *String_Builder) -> *String_Builder.Buffer`
- `get_buffer_data :: inline (using buffer: *String_Builder.Buffer) -> *u8`
- `simple_memcpy :: inline (using builder: *String_Builder, x: $T)`
- `expand :: (using builder: *String_Builder) -> bool`
- `append_and_steal_buffers :: (builder: *String_Builder, steal_from: *String_Builder)`
- `Temporary_Storage_Paged_Alloc :: struct`
- `Temporary_Storage_State :: struct`
- `Overflow_Page :: struct`
- `temporary_allocator_proc :: (mode: Allocator_Mode, requested_size: s64, old_size: s64, old_memory: *void, allocator_data: *void) -> *void`
- `auto_release_temp :: (loc := #caller_location) #expand`
- `get_temporary_storage_mark :: inline () -> Temporary_Storage_State`
- `set_temporary_storage_mark :: inline (state: Temporary_Storage_State, loc := #caller_location)`
- `reset_temporary_storage :: (_ts: *Temporary_Storage)`
- `free_pages_down_to :: (ts: *Temporary_Storage_Paged_Alloc, page: *Overflow_Page)`
- `ALIGNED_OVERFLOW_PAGE_ALLOCATION :: 32`
- `get_size_including_overflow_page_header :: (data_size: s64) -> s64`
- `add_new_page :: (ts: *Temporary_Storage_Paged_Alloc, minimum_size: s64) -> bool`
- `vis_init :: ()`
- `maybe_add_string :: (s: string)`
- `vis_update_if_it_has_been_long_enough :: ()`
- `vis_update :: ()`
- `vis_fini :: ()`
- `memory_visualizer_per_frame_update :: ()`
- `stb_print_float :: (builder: *String_Builder, format: FormatFloat) -> int`
- `SET_STORAGE_MARK_STAMP :: 0xda`
- `RESET_STORAGE_STAMP :: 0xdb`
- `FREE_PAGE_STAMP :: 0xdc`
- `Memory_Debugger :: struct`
- `Memory_Debugger :: REPLACEMENT_INTERFACE`
- `this_allocation_is_not_a_leak :: (memory: *void) #expand #no_debug`
- `this_allocation_leaks_todo :: (memory: *void) #expand #no_debug`
- `this_allocation_does_not_propagate_leak_ownership :: (memory: *void) #expand #no_debug`
- `memory_visualizer_per_frame_update :: () #expand #no_debug`
- `get_command_line_arguments :: () -> [] string`
- `assert_helper :: (message := "", args: .. Any, loc := #caller_location) -> bool`
- `assert :: (arg: bool, message := "", args: .. Any, loc := #caller_location) #no_debug #expand`
- `assert :: (#discard arg: bool, #discard message := "", #discard args: .. Any, #discard loc := #caller_location) #expand`
- `alloc :: (size: s64) -> *void`
- `free :: (memory: *void)`
- `realloc :: (memory: *void, size: s64, old_size: s64) -> *void`
- `create_heap :: (a: Allocator) -> Allocator`
- `destroy_heap :: (b: Allocator)`
- `get_capabilities :: (b: Allocator) -> (caps: Allocator_Caps, name: string)`
- `alloc_string :: (count: int) -> string`
- `talloc_string :: (count: int) -> string`
- `free :: inline (s: string)`
- `push_allocator :: (allocator: Allocator) #expand #no_debug`
- `push_allocator :: (proc: Allocator_Proc, data: *void) #expand #no_debug`
- `get_field :: (info: *Type_Info_Struct, name: string) -> *Type_Info_Struct_Member, offset_in_bytes: s64`
- `min :: (a: $T, b: T) -> T`
- `max :: (a: $T, b: T) -> T`
- `Min :: min`
- `Max :: max`
- `IsScalar :: (T: Type) -> bool #compile_time`
- `max :: inline (a: $T, x: ..T) -> T #modify`
- `min :: inline (a: $T, x: ..T) -> T #modify`
- `clamp :: (x: $T, a: T, b: T) -> T #modify`
- `swap :: inline (a: *$T, b: *T)`
- `enum_names :: ($t: Type) -> [] string`
- `enum_values_as_s64 :: ($T: Type) -> [] s64`
- `enum_values_as_enum :: ($T: Type) -> [] T`
- `enum_highest_value :: ($t: Type) -> s64`
- `enum_range :: ($t: Type) -> (low: s64, high: s64)`
- `enum_range_given_info :: (info: *Type_Info_Enum) -> (low: s64, high: s64)`
- `talloc :: inline (size: s64) -> *void`
- `temporary_alloc :: inline (size: s64) -> *void`
- `temp :: Allocator.{temporary_allocator_proc, null}`
- `temporary_allocator :: temp`
- `reset_temporary_storage :: ()`
- `Initialize :: (memory: *$T)`
- `New :: ($T: Type, $initialized := true) -> *T`
- `NewArray :: (count: s64, $T: Type, $initialized := true, alignment: s32 = -1) -> (result: [] T, unaligned_base: *void)`
- `align_forward :: (orig_size: s64, alignment: s64) -> s64`
- `pack_stack_trace :: () -> [] Stack_Trace_Node`
- `pack_stack_trace :: (node: *Stack_Trace_Node) -> [] Stack_Trace_Node`
- `print_stack_trace :: (node: *Stack_Trace_Node, prefix: string = "", to_standard_error := true)`
- `log_stack_trace :: (node: *Stack_Trace_Node = null, log_flags := Log_Flags.ERROR, prefix := "")`
- `get_stack_trace_strings :: (node: *Stack_Trace_Node = null) -> [..] string`
- `remember_allocators :: (data: *$T)`
- `resizable :: (a: [] $T, set_allocators := true) -> [..] T`
- `get_integer_range :: ($T: Type) -> (low: T, high: T)`
- `clamp_to_another_integer_type :: (value: $SourceType, $TargetType: Type) -> TargetType`
- `is_integer_type :: ($T: Type) -> (is_integer: bool, is_signed: bool, bits: u8)`
- `seconds_since_init :: () -> float64`
- `string_to_float :: (arg: string) -> (result: float, success: bool, remainder: string)`
- `string_to_float_new :: (arg: string) -> (success: bool, result: float, remainder: string)`
- `string_to_float64 :: (_s: string) -> (result: float64, success: bool, remainder: string) #no_abc`
- `string_to_float64_new :: (_s: string) -> (success: bool, result: float64, remainder: string) #no_abc`

### Linux and Android-specific declarations

- `get_working_directory :: () -> result: string`

### macOS-specific declarations

- `store_in_integer :: (str: string) -> s32`
- `get_working_directory :: () -> string`
- `IID :: struct`
- `LPVOID :: *void`
- `AbsoluteTime :: u64`
- `_NSGetExecutablePath :: (buf: *u8, bufsize: *u32) -> s32 #foreign libc`
- `NSSymbol :: *void`
- `NSIsSymbolNameDefined :: (symbolName: *u8) -> bool #foreign libc #deprecated "Use dlsym() instead."`
- `NSLookupAndBindSymbol :: (symbolName: *u8) -> NSSymbol #foreign libc #deprecated "Use dlsym() instead."`
- `NSAddressOfSymbol :: (symbol: NSSymbol) -> *void #foreign libc #deprecated "Use dlsym() instead."`
- `FourCharCode :: u32`
- `OSStatus :: s32`
- `OSType :: FourCharCode`
- `noErr : OSStatus`

### Linux, Android, and macOS POSIX declarations

- `sleep_milliseconds :: (milliseconds: s32)`
- `set_working_directory :: (s: string) -> bool`
- `exit :: (status: s32)`

### Windows-specific declarations

- `sleep_milliseconds :: (milliseconds: s32)`
- `set_working_directory :: (s: string) -> bool`
- `get_working_directory :: () -> result: string`
- `exit :: (status: s32)`

`sleep_milliseconds`, `set_working_directory`, and `exit` are present in the
Windows, Linux, Android, and macOS declarations. Linux and Android
load the POSIX declarations and add their own `get_working_directory`; macOS
loads the same POSIX declarations and adds its macOS `get_working_directory`;
Windows declares all four directly. The `set_working_directory` declaration
is not from the `Compiler` module. On POSIX it returns true exactly when
native `chdir` succeeds. On Windows it converts the UTF-8 path to wide
characters and returns false if conversion fails; otherwise it returns
whether `SetCurrentDirectoryW` succeeds. Neither variant reports a diagnostic
or returns the previous directory. `get_working_directory` returns the empty
string when the system cannot name the directory (on POSIX, one deleted while
the program was in it); there is nothing to free then.

Do not use `Runtime_Support.OS_IS_UNIX` as the availability condition for
these Basic procedures: that exported classification includes iOS and PS5
but excludes Windows. The Basic declarations establish the four targets
above; iOS has no corresponding Basic platform branch in the public
surface. The root Basic declarations name PS5 and Nintendo Switch platform
branches, but their procedure surface is unspecified, so availability
on those targets is unspecified.

- [CPU == .X64] `test_unsigned_division :: (random_iterations := 0, random_get : () -> u64 = null)`

## Public type members

#### `Time_Zone` (enum)

- `UTC   :: 0`
- `LOCAL :: 1`

#### `Calendar_Time` (struct)

- `year:        s32`
- `month_starting_at_0: s8`
- `day_of_month_starting_at_0: s8`
- `day_of_week_starting_at_0: s8`
- `hour:        s8`
- `minute:      s8`
- `second:      s8`
- `millisecond: s16`
- `time_zone: Time_Zone`

#### `S128` (struct)

- `low:  u64`
- `high: s64`

#### `U128` (struct)

- `low:  u64`
- `high: u64`

#### `Procedure_To_Elide` (struct)

- `procedure_name: string`
- `module_name:    string`

#### `Leak_Report_Options` (struct)

- `known_allocation_leaves: [] Procedure_To_Elide`
- `roots_to_skip:           [] Procedure_To_Elide`
- `group_by_these_calls:    [] Procedure_To_Elide`
- `scan_global_memory_to_remove_leaks := true`
- `scan_leaks_to_combine              := true`

#### `Leak_Report` (struct)

- `marked_as_non_leak_allocations: s64`
- `marked_as_non_leak_bytes:       s64`
- `sorted_summaries: [] *Leak_Summary`
- `leak_combiner_table: Table(*void, Leak_Combiner_Record)`
- `pool: Pool`

#### `Leak_Summary` (struct)

- `count: s64`
- `bytes: s64`
- `allocations_since_last_visualizer_update: s64`
- `bytes_allocated_since_last_visualizer_update: s64`
- `alloc_site_trace: *Packed_Stack_Trace`
- `allocator_proc: Allocator_Proc`
- `group_info: *Group_Info`
- `child_count: s64`
- `child_bytes: s64`

#### `Group_Info` (struct)

- `procedure: Procedure_To_Elide`
- `sub_summaries: [] *Leak_Summary`
- `root_nodes_in_common: s64`
- `leaf_nodes_in_common: s64`

#### `Formatter` (struct)

- `value   : Any`

#### `FormatInt` (struct)

- `using #as formatter : Formatter`

- `base             := 10`
- `minimum_digits   := 1`
- `padding          : u8 = #char "0"`
- `digits_per_comma : u16 = 0`
- `comma_string     := ""`

#### `FormatFloat` (struct)

- `#as using formatter : Formatter`

- `width := -1`
- `trailing_width := -1`
- `Zero_Removal :: enum { YES; NO; ONE_ZERO_AFTER_DECIMAL; }`
- `zero_removal := Zero_Removal.YES`
- `Mode :: enum { DECIMAL; SCIENTIFIC; SHORTEST; }`
- `mode := Mode.DECIMAL`
- `thousand_separators := false`
- `thousand_separator_character : u8 = #char ","`
- `decimal_separator_character  : u8 = #char "."`
- `exponent_character           : u8 = #char "e"`
- `Positive_Number_Prefix :: enum { NONE; PLUS; SPACE; }`
- `positive_number_prefix := Positive_Number_Prefix.NONE`

#### `FormatStruct` (struct)

- `#as using formatter : Formatter`

- `draw_type_name   := false`
- `use_long_form_if_more_than_this_many_members := 5`
- `separator_between_name_and_value := " = "`
- `short_form_separator_between_fields := ", "`
- `long_form_separator_between_fields  := "; "`
- `begin_string := "{"`
- `end_string   := "}"`
- `indentation_width : s32 = 4`
- `use_newlines_if_long_form := false`

#### `FormatArray` (struct)

- `#as using formatter : Formatter`
- `separator    := ", "`
- `begin_string := "["`
- `end_string   := "]"`
- `printing_stopped_early_string := "..."`
- `draw_separator_after_last_element := false`
- `stop_printing_after_this_many_elements := 100`

#### `Print_Style` (struct)

- `default_format_int:    FormatInt`
- `default_format_float:  FormatFloat`
- `default_format_struct: FormatStruct`
- `default_format_array:  FormatArray`
- `default_format_absolute_pointer: FormatInt`
- `struct_printer: (builder: *String_Builder, any: Any, struct_printer_data: *void) -> bool`
- `struct_printer_data: *void`
- `indentation_depth: s32`
- `state_flags: enum u32 { INSIDE_DATA_STRUCTURE_PRINTING :: 0x1; }`
- `log_runtime_errors := true`

`Basic` contributes `print_style: Print_Style` to the current context. Callers can configure the current context's default formatting and custom struct-printer hook through this member; it is a context field, not a standalone `Basic.print_style` global.

#### `String_Builder` (struct)

- `Buffer` (nested struct): `count: s64`, `allocated: s64`, optionally `ensured_count: s64` when `ENABLE_ASSERT` is true, then `next: *Buffer`. Its character storage follows the header in memory and is not a pointer field.
- `initialized            := false`
- `failed                 := false`
- `subsequent_buffer_size := -1`
- `allocator:             Allocator`
- `current_buffer:        *Buffer`
- `initial_bytes:         [STRING_BUILDER_BUFFER_SIZE] u8`

#### `Temporary_Storage_Paged_Alloc` (struct)

- `#overlay (implementation_data) using derived: struct {`
- `#as using base: Temporary_Storage;`

```jai
#overlay (implementation_data) using derived: struct {
    overflow_allocator: Allocator;
    overflow_pages: *Overflow_Page;
    original_data: *u8;
    original_size: s64;
    current_page_bytes_occupied: s64;
}
```

#### `Temporary_Storage_State` (struct)

- `top_overflow_page: *Overflow_Page`
- `current_page_bytes_occupied: s64`
- `total_bytes_occupied: s64`

#### `Overflow_Page` (struct)

- `next: *Overflow_Page`
- `allocator: Allocator`
- `size: s64`

The embedded `Temporary_Storage` base has `data: *u8`, `size: s64`, `total_bytes_occupied: s64`, `high_water_mark: s64`, `last_set_mark_location: Source_Code_Location`, and `implementation_data: [256]u8`.

#### `Memory_Debugger` (struct)

- `check_alloc   :: _check_alloc`
- `check_free    :: _check_free`
- `check_create_heap  :: _check_create_heap`
- `check_destroy_heap :: _check_destroy_heap`

#### `IID` (macOS-specific empty struct)

```jai
IID :: struct {}
```
This declaration has no fields in the supplied contract and is used as an opaque compatibility marker.

## Behavioral contract and constraints

#### Dynamic arrays and views

`array_view(array, index)` and `array_view(array, index, count)` produce non-owning slices into the source storage. A start at or beyond the current count yields an empty slice; an end beyond the source count is clamped. The two-argument form covers the suffix. Mutating elements through the view changes the original array. Any operation that reallocates or frees the owning array invalidates the view and pointers into it.

`array_add(array, value)` appends one value; the pointer-returning overload appends a new slot, initializes it through the element initializer when present, otherwise zero-initializes it, and returns a pointer invalidated by later growth. Appending a slice copies its elements into the array. Growth occurs when count reaches capacity and may move backing storage; reserve requests the stated capacity and does not change count. Allocation failure is asserted by the array growth path. `array_resize` sets the logical count; growing initializes new elements by initializer/zero-fill when `initialize` is true, and leaves new storage uninitialized when false. Shrinking only reduces logical count. `array_reset_keeping_memory` sets count to zero while retaining storage; `array_reset` releases storage and resets pointer/capacity/count.

`array_insert_at` accepts indexes from zero through count, shifts the suffix right, and preserves order. Ordered removal shifts the suffix left; unordered removal replaces the removed slot with the last element and does not preserve order. Both index removals require an in-range index and reduce count without clearing the vacated storage. `array_pop` returns and removes the last element; it requires a nonempty array. `array_find` returns the first matching index or `(false,-1)`. `array_add_if_unique` appends only if no match exists and reports whether insertion happened plus the resulting index. Value-removal helpers using unordered removal can remove several matches; ordered-by-value removes the first match.

#### Temporary storage

`temporary_allocator` is per-frame/per-context scratch storage. Individual frees are ignored; resizing within the already reserved padded allocation returns the same pointer, while growth obtains a new temporary block and copies the common prefix. Results allocated there must not outlive the corresponding temporary-storage mark/scope. `get_temporary_storage_mark` and `set_temporary_storage_mark` delimit a region; restoring a mark invalidates allocations made after it and releases overflow pages above that mark. `auto_release_temp` captures and restores a mark automatically. If the active context has no temporary storage, allocation falls back to the context allocator (avoiding recursion if that is itself the temporary allocator). This allocator is not thread-safe and does not support heap creation/destruction.

#### Logging and reflected enum views

`log(format_string, args..., loc := #caller_location, flags := Log_Flags.NONE, user_flags: u32 = 0, section: *Log_Section = null)` formats a message into temporary storage and passes it, `context.logger_data`, and a `Log_Info` containing the current context source identifier, location, flags, user flags, and section to `context.logger`. The section-first overload forwards to the same operation. `log_error` ORs `Log_Flags.ERROR` into the supplied flags and otherwise preserves the same fields. Basic does not filter verbose flags against `context.log_level`, append a newline, or substitute another logger. Empty formatted messages are not dispatched. A null logger is not checked before invocation; callers must provide a callable logger. Message bytes are temporary-storage-backed and are not guaranteed valid after that storage is reset. Formatting/allocation failure behavior beyond the builder's own failure handling is unspecified.

`enum_names(T)` and `enum_values_as_s64(T)` return the name and signed-64 value arrays from the type's reflection metadata, in declared reflection order; these are views of reflection storage, not owned copies. `enum_values_as_enum(T)` preserves that order but materializes enum-typed values in temporary storage, so its result expires when the relevant temporary storage is reset. `enum_range(T)` and `enum_range_given_info(info)` return the minimum and maximum reflected values as `s64`; an enum with no values returns `(0, 0)`. `enum_highest_value(T)` returns the high endpoint (thus zero for an empty enum). These helpers expose the metadata's `s64` value representation, including for enums whose underlying type is unsigned.

`get_field(info, name)` searches the given struct's direct members first, then recursively searches members marked `USING` whose types are structs. It returns the first depth-first match and the accumulated byte offset from the original struct. A miss returns `(null, 0)`. It does not search non-struct using members.

`IsScalar(T)` is a compile-time predicate true for integer, floating-point, pointer, and enum types; it is false for booleans and other type kinds. Variadic `min(a, x...)` and `max(a, x...)` require this predicate and a single common type `T`; they compare using `<`, include the leading `a`, and return `a` when no trailing values are supplied. This predicate is also the scalar-kind guard used by `clamp`.

#### Builder output and byte helpers

`is_alpha(c)` recognizes only ASCII English letters (`A`–`Z`, `a`–`z`);
`is_digit(c)` recognizes `0`–`9`; and `is_alnum(c)` is true for either of
those classes **or underscore**. These routines operate on one `u8` byte,
are locale-independent, and do not decode UTF-8. In particular, byte `95`
(`_`) gives `is_alpha = false` and `is_alnum = true`; byte `65` (`A`) gives
true for both; byte `48` (`0`) gives `is_alpha = false`, `is_digit = true`,
and `is_alnum = true`.

`print_sequence_to_builder(builder, args...)` appends each item in order using the default item-printing mode, with no separator inserted between items. Because that mode treats items as array/struct contents, string arguments are quoted. `print_bytes(a, length, format_string := "% ")` treats `a` as a byte pointer; each byte is passed as an unsigned integer to `print(format_string, formatInt(byte, base=16))`, then a newline is printed to standard output. The default therefore emits lowercase hexadecimal digits followed by a space for each byte and one final newline (for example, bytes `04 AB` produce `4 ab ` followed by a newline). The supplied format string can change each byte's formatting; this helper does not accept a destination builder or output stream option.

`consume_u8_and_length(input, destination, length)` returns false without changing `input` when `length` is negative or exceeds `input.count`. A zero length succeeds without copying or advancing. Otherwise it copies exactly `length` bytes from the front of the input string into `destination`, advances `input.data` by that amount, and subtracts the amount from `input.count`. It does not append a terminator. The destination must point to writable storage of at least the requested length.

#### Allocator and array ownership helpers

`get_capabilities(allocator)` invokes the allocator procedure with mode `CAPS`, zero sizes, the address of a `string` result as the `old_memory` argument, and null `allocator_data`. The procedure's single `*void` return is interpreted as the `Allocator_Caps` bitmask; the allocator writes the name string through that out pointer. Basic returns both values without copying the name, so its storage and lifetime are supplied by the allocator. The name-out argument and flags return are both part of the public allocator contract.

`create_heap(a)` sends `CREATE_HEAP` to `a.proc` and returns an allocator containing that same procedure and the returned data pointer. Under the allocator protocol, `CREATE_HEAP` creates a new heap independently of the input allocator data; the new data handle is used for subsequent allocation operations. `destroy_heap(b)` sends `DESTROY_HEAP` with `b.data`; after destruction, memory operations on that heap are errors. These wrappers do not query capabilities or turn an unsupported allocator into a heap-capable allocator. `CREATE_HEAP` without the advertised capability is an allocator error, so callers must check `CREATE_HEAP` first. The capability set has no separate `MULTIPLE_HEAPS` bit; availability of multiple heap handles is indicated by `CREATE_HEAP`. This does not guarantee parent/child heap nesting or cross-heap free behavior.

`remember_allocators(data)` stores an allocator in the pointed-to value's `allocator` field. It uses `context.allocator` when that allocator has a procedure, otherwise `context.default_allocator`. It does not recursively visit members or allocate/copy `data`.

`Preamble()` is an expansion intended for allocator bookkeeping procedures and requires a caller-local `allocator` value. During cross-compilation it returns immediately. It also returns without bookkeeping for pool, flat-pool, or temporary allocators, or while `context.inside_memory_debugger` is already true. Otherwise it sets that context flag, defers restoring it, temporarily switches `context.logger` to the runtime default logger, defers restoring the prior logger, and initializes the debugger on first use. This prevents recursive accounting and avoids logger lock inversion while debugger errors are reported. Its scoped restoration is part of the expansion contract.

`resizable(slice, set_allocators := true)` creates a dynamic-array header over the same backing pointer and count, with capacity initially equal to count; it does not copy or grow the elements. It consequently treats the backing storage as owned by the returned array, and any later reallocation/free through that array affects the original storage; callers must not separately free the original slice storage. Since a slice carries no allocator identity, the default records the current allocator (or default allocator fallback). With `set_allocators = false`, the returned allocator fields are left uninitialized and must be set before any operation that allocates or frees through the array.

The `array_resize(*[]T, new_count, initialize := true)` overload reallocates slice storage through the current allocator, updates the slice count, and initializes newly added elements when requested; shrinking truncates the logical slice and backing allocation. The `array_resize(*[..]T, ...)` overload reserves capacity if needed and changes the count; it does not shrink capacity when the count is reduced. Both initialize newly exposed elements through their type initializer when `initialize` is true; when false, new element contents are unspecified. `new_count` must be nonnegative. The allocator and array-growth failure behavior follows the underlying `realloc`/reserve path.

#### Stack-trace helpers

`pack_stack_trace()` copies the active context's linked stack trace; `pack_stack_trace(node)` copies from the supplied node. Both return a contiguous array of copied nodes whose `next` links point to the next array element, preserving the original traversal order and terminating at null. Node metadata pointers remain references to the original `Stack_Trace_Procedure_Info`; the returned array owns only its node storage and must be released by the caller with the matching array cleanup operation. The zero-argument form snapshots the current trace so its frame chain survives unwinding.

`print_stack_trace(node, prefix := "", to_standard_error := true)` writes one line for each node with metadata, in linked-list order, in the form `'procedure' at file:line`, prefixed by `prefix` and terminated by a newline. It uses low-level runtime output rather than the Basic logger or allocator. `get_stack_trace_strings(node := null)` returns one formatted string per metadata-bearing node, using the current context trace when `node` is null; both its array storage and strings use temporary storage. `log_stack_trace(node := null, log_flags := Log_Flags.ERROR, prefix := "")` uses the same default trace selection and logs each returned line with `prefix` and a newline under `log_flags`.

#### Memory-debugger reports

`allocations_before_this_call_will_not_be_considered_leaks()` establishes the allocation-index baseline used by subsequent leak reports. `this_allocation_is_not_a_leak` and its `this_allocation_leaks_todo` alias mark a known tracked allocation outside leak totals; null is ignored, while an unknown non-null address is logged as misuse and exits with status 1. `this_allocation_does_not_propagate_leak_ownership` disables propagation of reachable allocations from that tracked allocation; null and unknown addresses are ignored. These operations apply only when memory debugging is enabled; the facade's documented macros do nothing when it is disabled, but the argument expression is still evaluated.

Only allocations that are marked, that predate the baseline or that global memory points to count in `marked_as_non_leak_allocations` and `marked_as_non_leak_bytes`; what they reach is left out of the leaks without being counted. A leaking allocation still lists as its child any allocation it points to that is merely reachable from a non-leak, but never one that is itself marked or predates the baseline.

`make_leak_report(options := .{}, for_visualizer := false)` returns a report whose pool owns the report tables, summaries, and packed trace data; call `deinit(*report)` once to release it. Its summary list is ordered by descending total bytes including aggregated child bytes. Each summary reports allocation count/bytes, allocation-site trace, allocator procedure when unambiguous, optional grouping information, and child totals. The `known_allocation_leaves`, `roots_to_skip`, and `group_by_these_calls` option lists affect displayed trace trimming and grouping; the two scan flags control global-memory reachability cleanup and leak-combination scanning. With `for_visualizer = true`, both scans are skipped and the report includes allocations independent of the ordinary report baseline. Report construction provides a consistent diagnostic snapshot and may block concurrent bookkeeping; it is not intended as a latency-sensitive operation. Report-owned table/summary storage must remain valid for the lifetime of `report` and be released by its `deinit`. The private allocation organization is unconstrained provided report-owned resources retain that lifetime and are released by `deinit`.

`Procedure_To_Elide` matching uses the exact procedure name in `Stack_Trace_Procedure_Info.name`. When `module_name` is nonempty, the match additionally requires that its text occur as a substring of `Stack_Trace_Procedure_Info.location.fully_pathed_filename`; the runtime metadata has no module-identity field and no module lookup is performed. This is a case-sensitive path-fragment filter, not an exact module-name match. It applies to all three option lists, including grouping. To support module-filtered matching, stack-trace procedure metadata must provide both the exact procedure name and a source location with its fully pathed filename; a module name or module ID is not needed or consumed. The procedure address does not participate in matching. If symbols/locations are unavailable, these filters cannot match.

Trace trimming treats packed node index zero as the leaf and the final index as the root. `roots_to_skip` removes only the consecutive matching suffix at the root end (null metadata nodes at that end are also skipped); trimming stops at the first nonmatching root. `known_allocation_leaves` searches upward from the leaf and begins the displayed trace at the first matching node, including that matched node; intermediate frames between the allocation and the match are removed. If no leaf entry matches, the leaf end is unchanged. Grouping selects matching call nodes using the same name/path predicate; it does not supply a separate module identity mechanism.

`log_leak_report(report, options := .{})` logs one section per summary with its trace, followed by aggregate leak and marked-non-leak totals. `report_memory_leaks(options := .{})` builds, logs, and releases a report in one call. The report APIs are meaningful only when the memory debugger is enabled; details of the global-root scanner and ownership-combination heuristics are implementation policy and are not a stable graph-analysis contract.

#### Memory visualization hooks

When `MEMORY_DEBUGGER` and visualization support are enabled, `vis_init()` attempts to connect to the memory-visualization channel, retrying no more often than every 500 ms and logging the first connection failure. A successful connection begins by sending a reset/version message. `maybe_add_string(s)` queues a string only if it has not previously been queued. `vis_update()` sends pending string, procedure, trace, allocation, and free information for the current snapshot; a failed channel reservation leaves unsent queue entries available for a later update. `vis_update_if_it_has_been_long_enough()` initializes as needed and sends at most once per 250 ms, unless the caller has already selected explicit per-frame updates. `memory_visualizer_per_frame_update()` selects that explicit mode and immediately updates when connected. `vis_fini()` disconnects an active channel. The channel protocol is defined by the memory-visualization protocol module; Basic exposes no separate rendering API.

#### Formatting and calendar values

`to_float64_seconds(input)` returns a floating approximation of the signed
Apollo duration in seconds. It returns `(0, false)` when the whole-second
conversion cannot fit its signed 64-bit intermediate. For values that fit,
the fractional femtoseconds contribute to the result: positive and negative
one-femtosecond inputs correspond to nonzero results of approximately
`+1e-15` and `-1e-15` seconds, respectively. The API does not promise exact
representation of every femtosecond or a particular rounding mode for values
whose magnitude exceeds float64 precision; `success` reports range conversion,
not exactness. A result of zero with `success = true` for a one-femtosecond
input does not match the documented behavior.

`FormatInt`, `FormatFloat`, and `FormatStruct` embed the public `Formatter.value: Any` field through the declared `using` relationship. This field carries the value to print; factory helpers such as `formatInt`/`formatFloat`/`formatStruct` populate it alongside formatting options. Integer formatting defaults to base 10, minimum one digit, zero padding, and no grouping; float formatting defaults to decimal mode and removes redundant trailing zeroes; struct formatting chooses short form through five members and uses the configured separators/braces. Printing a formatter with a value of an incompatible kind can report a runtime formatting error; `Print_Style.log_runtime_errors` controls whether such errors are logged.

`Calendar_Time` uses zero-based `month_starting_at_0` (0–11), `day_of_month_starting_at_0` (0–30), and `day_of_week_starting_at_0` (Sunday=0 through Saturday=6); it has no one-based month/day representation. Add one to the first two fields for conventional human numbering. `calendar_to_string` returns an English string in `day Month year, hh:mm:ss` form, with a two-digit minimum for each clock component; it clamps the month to January–December and does not include milliseconds or a zone. `calendar_to_iso_string` returns `year-MM-DDThh:mm:ss.mmmZ` for UTC and the same string without a suffix for LOCAL; it zero-pads month/day/clock fields to at least two digits and milliseconds to at least three. It does not calculate or append a local offset. Both results use temporary storage.

`to_calendar(at, time_zone := .UTC)` and `calendar_to_apollo` use platform-native date conversion. The input `day_of_week_starting_at_0` is not used to determine the date. On POSIX, local conversion delegates daylight-saving selection and normalization to the C runtime with automatic DST selection: ambiguous and nonexistent local times have no cross-platform resolution guarantee, and invalid calendar components may be normalized instead of rejected. If the resulting seconds equal `-1`, `calendar_to_apollo` returns the zero `Apollo_Time` value; this also collides with the valid instant one second before the Unix epoch. For an unrepresentable input to POSIX `to_calendar`, the routine logs an error and returns a zero-initialized `Calendar_Time`. On Windows, invalid input conversion is asserted rather than reported by a success result; out-of-range Apollo input is logged, and subsequent native conversion may assert. Windows local conversion uses the system's current local-time rules. The module does not guarantee a policy for ambiguous/nonexistent local times or a minimum Windows release for these calendar calls; callers needing uniform validation or DST policy must perform it themselves.

`ConvertToApollo(m, FACTOR)` is a `#expand` macro with no procedure result type. Its expansion computes the scaled `Apollo_Time` value and uses a caller-scoped `` `return `` to return that value from the containing procedure; the call therefore belongs in a procedure whose return type accepts `Apollo_Time`. This is the same caller-return behavior demonstrated by the language's `#expand` examples. No leap-second representation is provided by `Calendar_Time`.

`Apollo_Time` is declared as an `#type,isa S128`; its operators are the inherited `S128` overloads, not Apollo_Time-specific `Add`/`Sub` procedures. `U128` also exposes subtraction. The Basic allocation helpers `alloc(size)` and `New(T, initialized := true)` use `context.allocator`; neither has a named allocator argument. The exported swap helper is lowercase `swap(a, b)` taking two pointers; Basic does not export `Swap`, `bubble_sort`, or a `get_time` function in this interface. The current `free` overloads accept `*void` and `string`.

`Apollo_Time` inherits its arithmetic/comparison operators from its `#type,isa
S128` base; there are no Apollo_Time-specific operator declarations. The
available signed operations are equality, logical-not/zero test, addition,
binary and unary subtraction, shifts by compile-time `u8`, signed division by
`S128` or `s64`, multiplication by `S128` or `s64`, and the four signed
ordering comparisons. Their exact `S128` signatures are listed above. They
operate on the two's-complement integer representation measured in
femtoseconds; subtraction of two times therefore yields the signed difference
in femtoseconds. Arithmetic operators return values without an overflow status;
fixed-width results discard overflow bits. Division by zero asserts. Shift
counts are `u8` compile-time parameters; left shift discards bits shifted
beyond 128, and right shift is arithmetic (sign extending). No time-zone,
calendar, or leap-second adjustment is performed by these integer operators.

`seconds_since_init()` establishes its own baseline lazily: the first call captures `current_time_monotonic()` as the baseline, and each call returns the monotonic elapsed time from that instant in `float64` seconds. It is not anchored to process startup, Basic module initialization, or an explicit public initialization call. Its first-use initialization is not guaranteed thread-safe by the source; callers should arrange a single-threaded first call before concurrent use if they need a shared origin.

`to_s128(x)` accepts source types whose reflected kind is `.INTEGER`, including ordinary signed and unsigned integer types. It rejects distinct integer types, enums, floats, booleans, and `S128`/`U128` records through its compile-time `#modify` guard. The result's low half contains the no-check conversion of the source to `u64`; its high half is the sign extension determined by converting the source to `s64` and taking that value's sign. This is signed 64-bit sign extension, not a zero extension based on unsigned source metadata. `to_u128(x)` accepts exactly the built-in unsigned types `u8`, `u16`, `u32`, and `u64`; it places the value in the low half and zeroes the high half. Its guard rejects signed integers (including negative inputs), distinct integer types, enums, floats, booleans, and existing 128-bit records. These are compile-time admissibility rules; neither function returns an overflow status, and any conversion checks follow ordinary Jai cast semantics.

On x64 targets, `test_unsigned_division(random_iterations := 0, random_get := null)` is a public diagnostic exercise (the source scope is CPU-conditional, not Windows-conditional). It always runs its deterministic checks, prints each tested division result, and asserts quotient/remainder consistency. When `random_iterations` is positive, it additionally calls `random_get` three times per iteration to construct sample operands and asserts the same identities; the callback must be non-null, and its first returned value must be nonzero because the exercise takes a remainder by that value. No explicit iteration-count clamp is applied. With the default zero iterations, the callback is not used and may be null.

The `print(format_string, args...)` overload writes formatted bytes to standard output, or to standard error when `to_standard_error` is true, and returns the number of bytes written. The `print(builder, format_string, args...)` overload appends to the supplied `String_Builder` and returns whether formatting succeeded; `print_to_builder` is the same callable. `sprint` returns a string allocated for the caller's normal allocator context. `tprint` returns a string in temporary storage, so the value is valid only for the corresponding temporary-storage lifetime. `exit(status)` requests process termination with that status on the declared POSIX and Windows targets.

#### Format strings and argument selection

Ordinary format-string bytes are copied unchanged. `%` inserts an argument. A decimal integer immediately after `%` selects a one-based argument index; `%0` selects the next sequential argument. The sequential cursor starts at argument 1 and, after any successful insertion, advances to the argument after the one just selected. Thus a later sequential directive follows a positional directive: `sprint("%2 % %1", "A", "B", "C")` produces `"B C A"`. With only two arguments, the middle sequential insertion is out of range and omitted (with a runtime diagnostic when enabled), yielding `"B  A"`; after an invalid selection the cursor resets to argument 1. `%00` emits an empty string and consumes no argument. Byte `0x1f` anywhere in the format string emits a literal `%`; for example, `sprint("left\x1fright")` produces `"left%right"`. `%%` is not the literal-percent escape.

If the selected index is beyond the argument list, the insertion is omitted. The sequential cursor resets to argument 1 after such an invalid selection. When `Print_Style.log_runtime_errors` is true, the invalid index is logged; when false, it is silently omitted. These format errors do not by themselves make `print_to_builder` return false: that result reports builder failure. A trailing `%` is an ordinary sequential insertion, so it is diagnosed and omitted if no argument remains, or consumes the next argument when one does. A non-digit after `%` is left in the string and the marker consumes the next sequential argument; for example, `sprint("%-", "X")` produces `"X-"`. Decimal index overflow behavior is unspecified.

Top-level strings are appended without quotes. Strings printed inside arrays or structs are surrounded by double quotes, with line-feed bytes represented as `\\n`; other bytes are copied as-is. Booleans print as `true` or `false`. Integers and floats use the active `Print_Style` defaults unless wrapped in `FormatInt` or `FormatFloat`. A null pointer prints `null`; other pointers use `default_format_absolute_pointer` (hexadecimal with groups of four digits separated by `_` by default). A custom `Print_Style.struct_printer` may handle structs, pointers-to-structs, and variants before their default rendering.

Plain enums print their declared name for a matching value and a numeric out-of-range marker for an unmatched nonzero value; unmatched zero prints `0`. Flag enums print a matching zero-value name when declared, otherwise `0`; nonzero values print recognized flag names joined with ` | ` and then any remaining bits in hexadecimal. Variants print as their underlying value. A type value prints its type name, void prints `void`, code prints `Code`, overload sets print `(procedure overloads)`, and null procedures print `null`. An all-zero `Any` (both `type` and `value_pointer` null) prints `Any.{}`. A missing type with non-null storage is invalid; a valid scalar type with null storage faults when formatting consumes its payload. Neither partially null case is a promised diagnostic fallback. Merely boxing an unused or ignored null lvalue does not read it, including zero-sized values; valid boxed values retain borrowed storage identity.

`FormatInt.base` accepts 2 through 64; digits are selected in order from `0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_.`. Decimal signed values receive a leading minus sign and are formatted by magnitude; for other bases the stored integer bits are treated as unsigned. `minimum_digits` pads on the left to at least that many digits using `padding`, with a maximum effective count of 64. Grouping is enabled only when both `digits_per_comma` and `comma_string` are nonempty/nonzero; it groups from the right. Unsupported input kinds or invalid bases/minimums assert.

`FormatFloat.trailing_width` is the requested fractional precision; a negative value selects six digits. `width` is a minimum total character width, filled on the left with spaces. Modes are decimal, scientific, or shortest representation. Decimal and scientific output use the configured decimal/exponent bytes; optional thousands separators group the integer part in threes. `positive_number_prefix` adds either `+` or a leading space to nonnegative values. `zero_removal.YES` removes redundant fractional zeroes and the now-empty decimal separator; `ONE_ZERO_AFTER_DECIMAL` retains one fractional zero when the fraction is otherwise all zero; `NO` retains the requested trailing precision. With an explicit `trailing_width`, characters removed by zero removal are replaced with spaces after the number to preserve that fractional field width. Shortest mode chooses its own decimal/scientific form and removes redundant zeroes. NaN and infinity print as `NaN` and `Inf` (with a sign/prefix when applicable).

`FormatStruct` chooses short form when the count of printable members is at most `use_long_form_if_more_than_this_many_members`; a negative threshold always selects long form. Short form prints values only with `short_form_separator_between_fields`. Long form prints member names, then `separator_between_name_and_value` twice, and appends `long_form_separator_between_fields` after each member. Constants, imported, overlay, and zero-sized members are omitted. `draw_type_name`, delimiters, and optional newlines/indentation apply as configured. `FormatArray` prints its delimiters and elements up to `stop_printing_after_this_many_elements`; a nonnegative limit truncates and appends `printing_stopped_early_string` when elements remain. Separators appear between printed values, and after the final printed value only when `draw_separator_after_last_element` is true. Default array output uses brackets, comma-space separators, and a 100-element limit.

On macOS arm64, `formatFloat` on a nested `distinct float64` / `isa` numeric wrapper preserves the boxed scalar and, with `trailing_width = 2` and `zero_removal = .NO`, prints `8.50`; `formatInt` on a nested distinct `s64` wrapper with `base = 16` prints `29`. For the tested valid tagged choice, `FormatStruct` prints only the selected case as `{COUNT, 37}` or `{TEXT, chosen}`. The same active-case output is produced with the default threshold and with `use_long_form_if_more_than_this_many_members` set to one less than the active member count; the inactive union arm is not read. This covers valid layouts and values only; malformed tags and other union representations remain unspecified.

Passing a formatter with no value prints `(null Any)`. `FormatInt` requires an integer value and `FormatFloat` requires a float value. `FormatStruct` accepts a struct or pointer to struct; another value kind logs an error and prints `error`. Formatters are expanded only when passed as top-level arguments; nested formatter values print as ordinary struct values. An uninitialized `Any` prints `Any.{}`. Other unsupported or corrupt reflected values use the explicit fallback strings of their type case.

Other Basic facilities use the allocator/context supplied by their API or the active context default. OS-specific time and file operations are available only on their declared targets. Parsing and allocation failures follow each operation’s return/assert behavior; there is no single module-wide error convention.


Platform coverage note: exports for Nintendo and PS5 targets are unspecified.

## Portability notes

The standard library of this repository uses `BUILD_OS` and `align_of` as built-in names. The official compiler reports both as undeclared identifiers, so code that must also build there should not rely on them.
