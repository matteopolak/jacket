#import "Basic";
#import "String";

safe :: () -> int {
    source := cast(*u8) alloc(64);
    destination := cast(*u8) alloc(64);

    memset(source, 1, 64);
    memcpy(destination, source, 64);
    text := string.{ 64, source };

    find_index_from_left(text, #char "x");
    result := cast(int) source[0] + destination[0];

    free(source);
    free(destination);

    return result;
}

bad :: () -> u8 {
    value := cast(*u8) alloc(8);
    memset(value, 0, 8);
    free(value);
    return value[0];
}

fiber_header_survives :: () -> int {
    fiber: Fiber_Context;
    fiber_free(*fiber);
    return cast(int) fiber.sp;
}

Fiber_Context :: struct {
    sp: *void;
    stack_low: *void;
    stack_high: *void;
    stack_dealloc: *void;
}

fiber_free :: (fiber: *Fiber_Context) #intrinsic;

bad_write :: () {
    value := cast(*u8) alloc(8);
    free(value);
    memset(value, 0, 1);
}

safe_zero_write :: () {
    value := cast(*u8) alloc(8);
    free(value);
    memset(value, 0, 0);
}
