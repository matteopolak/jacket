#import "Basic";
#import "Math";

main :: () {
    check(Vector2.{ 2, -4 }, Vector2.{ 4, 2 }, Vector2.{ 8, -8 }, Vector2.{ 0.5, -2 }, Vector2.{ 4, -2 });
    check(Vector3.{ 2, -4, 8 }, Vector3.{ 4, 2, -2 }, Vector3.{ 8, -8, -16 }, Vector3.{ 0.5, -2, -4 }, Vector3.{ 4, -2, 1 });
    check(Vector4.{ 2, -4, 8, -16 }, Vector4.{ 4, 2, -2, -4 }, Vector4.{ 8, -8, -16, 64 }, Vector4.{ 0.5, -2, -4, 4 }, Vector4.{ 4, -2, 1, -0.5 });
    assert(Vector4.{ 1, 2, 3, 4 } * Vector4.{ 5, 6, 7, 8 } == Vector4.{ 5, 12, 21, 32 });
    assert(distance_squared(Vector4.{ 2, -4, 8, -16 }, Vector4.{ 4, 2, -2, -4 }) == 284);
    assert(unit_vector(Vector4.{ 0, 0, 0, 5 }) == Vector4.{ 0, 0, 0, 1 });
    assert(unit_vector(Vector4.{}) == Vector4.{});
    print("ok\n");
}

check :: (a: $T, b: T, product: T, quotient: T, reciprocal: T) {
    assert(a * b == product);
    assert(a / b == quotient);
    assert(8.0 / a == reciprocal);
    assert(a * 2.0 == 2.0 * a);
    assert(a / 2.0 == a * 0.5);
    assert(multiply(a, 2.0) == a * 2.0);
    assert((a + b) - b == a);
    assert(-(-a) == a);
    result := a;
    result *= b;
    assert(result == product);
    result = a;
    result /= b;
    assert(result == quotient);
}
