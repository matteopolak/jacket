#import "Basic";
H :: #import "Float16"(ASSERT_ON_OVERFLOW = true);

main :: () {
    H.float32_to_float16(70000.0);
    print("overflow assertion was missed\n");
}
