#import "Basic";

main :: () {
    k: s64 = 7;
    value := Any.{ type = null, value_pointer = *k };

    assert(value.type == null && value.value_pointer != null);
    print("%", value);
    print("unreachable\n");
}
