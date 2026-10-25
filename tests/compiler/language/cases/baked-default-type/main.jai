#import "Basic";

Stack :: struct($N: s64, $T: Type) {
    items: [N] T;
    count: s64;
}

push :: (stack: *Stack, $initialize := true) -> *stack.T {
    stack.count += 1;
    result := *stack.items[stack.count - 1];
    #if initialize result.* = .{};
    return result;
}

push :: (stack: *Stack, value: stack.T) {
    push(stack, false).* = value;
}

main :: () {
    stack: Stack(4, u32);

    push(*stack, 11);
    push(*stack, 22);
    assert(stack.count == 2 && stack.items[0] == 11 && stack.items[1] == 22, "an integer is a value, not the flag");
    push(*stack, false);
    assert(stack.count == 3, "a bool is the flag");
    print("ok\n");
}
