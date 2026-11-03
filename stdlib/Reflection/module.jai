// Helpers for working with type information at run time: integer ranges, struct members, enum names and values, and setting a value from text.
//
// ```jai
// Color :: enum {
//     RED;
//     GREEN;
//     BLUE;
// }
//
// assert(enum_value_to_name(Color.GREEN) == "GREEN");
//
// value: Color;
//
// assert(set_value_from_string(value, "BLUE"));
// assert(value == .BLUE);
// ```

get_array_count_and_data :: Impl.get_array_count_and_data;
signed_integer_range_from_size :: Impl.signed_integer_range_from_size;
unsigned_integer_range_from_size :: Impl.unsigned_integer_range_from_size;
range_check_and_store :: Impl.range_check_and_store;
get_struct_field_info :: Impl.get_struct_field_info;
enum_name_to_value :: Impl.enum_name_to_value;
enum_value_to_name :: Impl.enum_value_to_name;
get_enum_value :: Impl.get_enum_value;
set_enum_value :: Impl.set_enum_value;
set_value_from_string :: Impl.set_value_from_string;

#scope_file
Impl :: #import "internal/reflection";
