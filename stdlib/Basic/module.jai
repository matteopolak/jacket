#module_parameters()(MEMORY_DEBUGGER := false, ENABLE_ASSERT := true, REPLACEMENT_INTERFACE: $I/interface Memory_Debugger_Interface = Memory_Debugger_Interface, VISUALIZE_MEMORY_DEBUGGER := true, TEMP_ALLOCATOR_OVERWRITE_MEMORY := false) {
    #scope_file
    Memory_Debugger_Interface :: Interface_Definition.Memory_Debugger_Interface;
    Interface_Definition :: #import "internal/basic-interface";
    #scope_export
}

// Array algorithms are canonical declarations of Basic. Their generic bodies
// must see overloads explicitly injected into Basic, while private-provider
// generics retain their independent declaration environment.
#load "../internal/basic/allocator-call.jai";
#load "../internal/basic/arrays.jai";

STRING_BUILDER_BUFFER_SIZE :: Impl.STRING_BUILDER_BUFFER_SIZE;
String_Builder :: Impl.String_Builder;
get_base_buffer :: Impl.get_base_buffer;
get_current_buffer :: Impl.get_current_buffer;
get_buffer_data :: Impl.get_buffer_data;
init_string_builder :: Impl.init_string_builder;
ensure_contiguous_space :: Impl.ensure_contiguous_space;
ensure_contiguous_space_and_return_available_range :: Impl.ensure_contiguous_space_and_return_available_range;
advance_through_ensured_space :: Impl.advance_through_ensured_space;
append :: Impl.append;
append_by_pointer :: Impl.append_by_pointer;
simple_memcpy :: Impl.simple_memcpy;
expand :: Impl.expand;
append_and_steal_buffers :: Impl.append_and_steal_buffers;
builder_string_length :: Impl.builder_string_length;
free_buffers :: Impl.free_buffers;
reset :: Impl.reset;
builder_to_string :: Impl.builder_to_string;
write_builder :: Impl.write_builder;
assert_helper :: Impl.assert_helper;
assert :: Impl.assert;
print :: Impl.print;
min :: Impl.min;
max :: Impl.max;
clamp :: Impl.clamp;
swap :: Impl.swap;
to_upper :: Impl.to_upper;
to_lower :: Impl.to_lower;
is_alpha :: Impl.is_alpha;
is_digit :: Impl.is_digit;
is_alnum :: Impl.is_alnum;
is_space :: Impl.is_space;
is_print :: Impl.is_print;
is_integer_type :: Impl.is_integer_type;
IsScalar :: Impl.IsScalar;
get_integer_range :: Impl.get_integer_range;
clamp_to_another_integer_type :: Impl.clamp_to_another_integer_type;
enum_names :: Impl.enum_names;
enum_values_as_s64 :: Impl.enum_values_as_s64;
enum_values_as_enum :: Impl.enum_values_as_enum;
get_field :: Impl.get_field;
enum_range :: Impl.enum_range;
enum_range_given_info :: Impl.enum_range_given_info;
enum_highest_value :: Impl.enum_highest_value;
advance :: Impl.advance;
string_to_int :: Impl.string_to_int;
to_integer :: Impl.to_integer;
eat_spaces :: Impl.eat_spaces;
string_to_float64 :: Impl.string_to_float64;
string_to_float :: Impl.string_to_float;
string_to_float64_new :: Impl.string_to_float64_new;
string_to_float_new :: Impl.string_to_float_new;
alloc :: Impl.alloc;
get_capabilities :: Impl.get_capabilities;
Memory_Debugger :: Impl.Memory_Debugger;
Preamble :: Impl.Preamble;
Procedure_To_Elide :: Impl.Procedure_To_Elide;
Group_Info :: Impl.Group_Info;
Leak_Summary :: Impl.Leak_Summary;
Leak_Report_Options :: Impl.Leak_Report_Options;
Leak_Report :: Impl.Leak_Report;
allocations_before_this_call_will_not_be_considered_leaks :: Impl.allocations_before_this_call_will_not_be_considered_leaks;
this_allocation_is_not_a_leak :: Impl.this_allocation_is_not_a_leak;
this_allocation_leaks_todo :: Impl.this_allocation_leaks_todo;
this_allocation_does_not_propagate_leak_ownership :: Impl.this_allocation_does_not_propagate_leak_ownership;
add_allocation_to_table :: Impl.add_allocation_to_table;
do_free_bookkeeping :: Impl.do_free_bookkeeping;
add_allocation_this_frame :: Impl.add_allocation_this_frame;
add_free_this_frame :: Impl.add_free_this_frame;
make_leak_report :: Impl.make_leak_report;
log_leak_report :: Impl.log_leak_report;
report_memory_leaks :: Impl.report_memory_leaks;
deinit :: Impl.deinit;
_check_alloc :: Impl._check_alloc;
_check_free :: Impl._check_free;
_check_create_heap :: Impl._check_create_heap;
_check_destroy_heap :: Impl._check_destroy_heap;
create_heap :: Impl.create_heap;
destroy_heap :: Impl.destroy_heap;
memory_visualizer_per_frame_update :: Impl.memory_visualizer_per_frame_update;
vis_init :: Impl.vis_init;
vis_update :: Impl.vis_update;
vis_update_if_it_has_been_long_enough :: Impl.vis_update_if_it_has_been_long_enough;
maybe_add_string :: Impl.maybe_add_string;
vis_fini :: Impl.vis_fini;
free :: Impl.free;
realloc :: Impl.realloc;
alloc_string :: Impl.alloc_string;
copy_string :: Impl.copy_string;
to_string :: Impl.to_string;
to_c_string :: Impl.to_c_string;
Initialize :: Impl.Initialize;
New :: Impl.New;
align_forward :: Impl.align_forward;
get_u64_from_void_pointer :: Impl.get_u64_from_void_pointer;
get_s64_from_void_pointer :: Impl.get_s64_from_void_pointer;
print_item_to_builder :: Impl.print_item_to_builder;
Formatter :: Impl.Formatter;
FormatFloat :: Impl.FormatFloat;
FormatStruct :: Impl.FormatStruct;
FormatArray :: Impl.FormatArray;
Print_Style :: Impl.Print_Style;
formatFloat :: Impl.formatFloat;
formatStruct :: Impl.formatStruct;
FormatInt :: Impl.FormatInt;
formatInt :: Impl.formatInt;
print_integer :: Impl.print_integer;
print_float :: Impl.print_float;
stb_print_float :: Impl.stb_print_float;
append_indentation :: Impl.append_indentation;
append_struct_info :: Impl.append_struct_info;
print_array :: Impl.print_array;
isFormatter :: Impl.isFormatter;
print_type_to_builder :: Impl.print_type_to_builder;
type_to_string :: Impl.type_to_string;
print_u64_base_n :: Impl.print_u64_base_n;
print_to_builder :: Impl.print_to_builder;
my_is_digit :: Impl.my_is_digit;
parse_number :: Impl.parse_number;
sprint :: Impl.sprint;
tprint :: Impl.tprint;
print_sequence_to_builder :: Impl.print_sequence_to_builder;
print_bytes :: Impl.print_bytes;
consume_u8_and_length :: Impl.consume_u8_and_length;
log :: Impl.log;
log_error :: Impl.log_error;
pack_stack_trace :: Impl.pack_stack_trace;
get_stack_trace_strings :: Impl.get_stack_trace_strings;
print_stack_trace :: Impl.print_stack_trace;
log_stack_trace :: Impl.log_stack_trace;
print_character :: Impl.print_character;
Min :: min;
Max :: max;
push_allocator :: Impl.push_allocator;
S128 :: Impl.S128;
U128 :: Impl.U128;
U128_ERROR :: Impl.U128_ERROR;
S128_ERROR :: Impl.S128_ERROR;
multiply_64bits_with_overflow :: Impl.multiply_64bits_with_overflow;
to_u128 :: Impl.to_u128;
to_s128 :: Impl.to_s128;
logical_shift_right :: Impl.logical_shift_right;
unsigned_divide_with_remainder :: Impl.unsigned_divide_with_remainder;
signed_divide_with_remainder :: Impl.signed_divide_with_remainder;
#if CPU == .X64 test_unsigned_division :: Impl.test_unsigned_division;
Apollo_Time :: Impl.Apollo_Time;
APOLLO_TIME_INVALID :: Impl.APOLLO_TIME_INVALID;
APOLLO_TIME_FOR_JAN_1_1970 :: Impl.APOLLO_TIME_FOR_JAN_1_1970;
FEMTOSECONDS :: Impl.FEMTOSECONDS;
Time_Zone :: Impl.Time_Zone;
Calendar_Time :: Impl.Calendar_Time;
seconds_to_apollo :: Impl.seconds_to_apollo;
ConvertToApollo :: Impl.ConvertToApollo;
milliseconds_to_apollo :: Impl.milliseconds_to_apollo;
microseconds_to_apollo :: Impl.microseconds_to_apollo;
nanoseconds_to_apollo :: Impl.nanoseconds_to_apollo;
calendar_to_string :: Impl.calendar_to_string;
calendar_to_iso_string :: Impl.calendar_to_iso_string;
ConvertFromApollo :: Impl.ConvertFromApollo;
to_seconds :: Impl.to_seconds;
to_milliseconds :: Impl.to_milliseconds;
to_microseconds :: Impl.to_microseconds;
to_nanoseconds :: Impl.to_nanoseconds;
compare_apollo_times :: Impl.compare_apollo_times;
to_float64_seconds :: Impl.to_float64_seconds;
to_apollo :: Impl.to_apollo;
#if OS == .MACOS || OS == .LINUX || OS == .ANDROID || OS == .WINDOWS seconds_since_init :: Impl.seconds_since_init;

#if OS == .WINDOWS {
    to_calendar :: Impl.to_calendar;
    calendar_to_apollo :: Impl.calendar_to_apollo;
    filetime_to_apollo :: Impl.filetime_to_apollo;
    to_filetime :: Impl.to_filetime;
    current_time_consensus :: Impl.current_time_consensus;
    current_time_monotonic :: Impl.current_time_monotonic;
}

#if OS == .MACOS || OS == .LINUX || OS == .ANDROID {
    timespec_to_apollo :: Impl.timespec_to_apollo;
    to_timespec :: Impl.to_timespec;
    current_time_consensus :: Impl.current_time_consensus;
    current_time_monotonic :: Impl.current_time_monotonic;
    to_calendar :: Impl.to_calendar;
    calendar_to_apollo :: Impl.calendar_to_apollo;
}

Temporary_Storage_Paged_Alloc :: Impl.Temporary_Storage_Paged_Alloc;
Temporary_Storage_State :: Impl.Temporary_Storage_State;
Overflow_Page :: Impl.Overflow_Page;
ALIGNED_OVERFLOW_PAGE_ALLOCATION :: Impl.ALIGNED_OVERFLOW_PAGE_ALLOCATION;
SET_STORAGE_MARK_STAMP :: Impl.SET_STORAGE_MARK_STAMP;
RESET_STORAGE_STAMP :: Impl.RESET_STORAGE_STAMP;
FREE_PAGE_STAMP :: Impl.FREE_PAGE_STAMP;
temp :: Impl.temp;
temporary_allocator :: Impl.temporary_allocator;
talloc :: Impl.talloc;
temporary_alloc :: Impl.temporary_alloc;
talloc_string :: Impl.talloc_string;
copy_temporary_string :: Impl.copy_temporary_string;
temp_c_string :: Impl.temp_c_string;
temporary_allocator_proc :: Impl.temporary_allocator_proc;
get_size_including_overflow_page_header :: Impl.get_size_including_overflow_page_header;
add_new_page :: Impl.add_new_page;
free_pages_down_to :: Impl.free_pages_down_to;
get_temporary_storage_mark :: Impl.get_temporary_storage_mark;
set_temporary_storage_mark :: Impl.set_temporary_storage_mark;
auto_release_temp :: Impl.auto_release_temp;
reset_temporary_storage :: Impl.reset_temporary_storage;
NewArray :: Impl.NewArray;

#if OS == .WINDOWS || OS == .LINUX || OS == .ANDROID || OS == .MACOS {
    get_working_directory :: Impl.get_working_directory;
    set_working_directory :: Impl.set_working_directory;
    exit :: Impl.exit;
    sleep_milliseconds :: Impl.sleep_milliseconds;
}

get_command_line_arguments :: Impl.get_command_line_arguments;

// Reexport the canonical overload sets through ordinary visibility.
using,only(.["==", "!", "+", "-", "<", "<=", ">", ">="]) #import "internal/basic"(MEMORY_DEBUGGER = MEMORY_DEBUGGER, ENABLE_ASSERT = ENABLE_ASSERT, REPLACEMENT_INTERFACE = REPLACEMENT_INTERFACE, VISUALIZE_MEMORY_DEBUGGER = VISUALIZE_MEMORY_DEBUGGER, TEMP_ALLOCATOR_OVERWRITE_MEMORY = TEMP_ALLOCATOR_OVERWRITE_MEMORY);

#scope_file
Impl :: #import "internal/basic"(MEMORY_DEBUGGER = MEMORY_DEBUGGER, ENABLE_ASSERT = ENABLE_ASSERT, REPLACEMENT_INTERFACE = REPLACEMENT_INTERFACE, VISUALIZE_MEMORY_DEBUGGER = VISUALIZE_MEMORY_DEBUGGER, TEMP_ALLOCATOR_OVERWRITE_MEMORY = TEMP_ALLOCATOR_OVERWRITE_MEMORY);
