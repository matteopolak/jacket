// Run as a WASI command: prints its arguments, the target, some heap-heavy work, and exits
// with the number of arguments.
#import "Basic";
#import "String";
#import "Hash_Table";
#import "Sort";
#import "Math";

main :: () -> s32 {
    args := get_command_line_arguments();
    for args if it_index > 0 print("arg %: %\n", it_index, it);
    print("OS % CPU % pointer % bytes\n", OS, CPU, size_of(*void));

    // Many small blocks that grow, are freed and reused.
    table: Table(string, int);
    for 0..9999 table_add(*table, sprint("key%", it), it * 3);
    total := 0;
    for 0..9999 {
        found := table_find_pointer(*table, tprint("key%", it));
        if found total += found.*;
    }
    print("table % %\n", table.count, total);

    // One block that grows to several megabytes, past the initial memory.
    big: [..] s64;
    for 0..999_999 array_add(*big, (it * 7919) % 1000);
    quick_sort(big, (a, b) => a - b);
    print("sorted % % %\n", big.count, big[0], big[big.count - 1]);

    builder: String_Builder;
    for 0..499 print_to_builder(*builder, "% ", it);
    text := builder_to_string(*builder);
    print("text % %\n", text.count, split(text, " ").count);
    print("float % %\n", 3.25 * 2, sqrt(2.0));
    return cast(s32) args.count - 1;
}
