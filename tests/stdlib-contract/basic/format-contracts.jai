// Public format-string selection, formatted wrappers, and builder sequence behavior.
#import "Basic";

main :: () {
    assert(sprint("%2 % %1", "A", "B") == "B B A");
    assert(sprint("%00") == "");
    assert(sprint("left\x1fright") == "left%right");
    assert(sprint("%-", "X") == "X-");

    hex := formatInt(255, base = 16, minimum_digits = 4);

    assert(sprint("%", hex) == "00ff");

    builder: String_Builder;

    init_string_builder(*builder);
    assert(print(*builder, "value=%", 42));
    assert(builder_to_string(*builder) == "value=42");
    reset(*builder);
    print_sequence_to_builder(*builder, 1, true, "x");
    assert(builder_to_string(*builder) == "1true\"x\"");
    free_buffers(*builder);

    print("ok\n");
}
