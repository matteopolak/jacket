// Configuration review reproducer; root constants do not configure imported scopes.
#import "Basic";

ENABLE_ASSERT :: true;
#assert ENABLE_ASSERT;

main :: () {
    builder: String_Builder;
    buffer := get_current_buffer(*builder);
    assert(buffer.ensured_count == 0);
}
