#import "Basic";

main :: () {
    value := Any.{ type = type_info(s64), value_pointer = null };
    assert(value.type != null && value.value_pointer == null);
    print("%", value);
    print("unreachable\n");
}
