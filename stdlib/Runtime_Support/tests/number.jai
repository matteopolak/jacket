// Exact independent output for reviewed radix clamping and stdout/stderr choice.
R :: #import "Runtime_Support";

main :: () {
    R.write_nonnegative_number(0);
    R.write_string("|");
    R.write_nonnegative_number(0xffffffffffffffff);
    R.write_string("|");
    R.write_nonnegative_number(255, 16);
    R.write_string("|");
    R.write_nonnegative_number(10, 1);
    R.write_string("|");
    R.write_nonnegative_number(255, 99);
    R.write_string("|");
    R.write_number(-37);
    R.write_string("|");
    R.write_number(9223372036854775807);
    R.write_strings("|", "done", "\n");
    R.write_string_unsynchronized("unsynchronized\n");
    R.runtime_support_default_logger("ordinary\n", null, .{});
    R.write_number(-37, 16, true);
    R.write_string("|", true);
    R.write_nonnegative_number(7, 2, true);
    R.write_string("\n", true);
    info: Log_Info;
    info.common_flags = .ERROR;
    R.runtime_support_default_logger("error\n", null, info);
}
