#import "Basic"()(ENABLE_ASSERT = false);

main :: () {
    value := Any.{ type = type_info(s64), value_pointer = null };
    print("%", value);
    print("unreachable\n");
}
