// Vectors, quaternions and square matrices compare field by field with `==` and `!=`.
#import "Basic";
#import "Math";

main :: () {
    a := Vector2.{1, 2};
    assert(a == Vector2.{1, 2} && a != Vector2.{1, 3});
    b := Vector3.{1, 2, 3};
    assert(b == Vector3.{1, 2, 3} && b != Vector3.{1, 2, 4});
    c := Vector4.{1, 2, 3, 4};
    assert(c == Vector4.{1, 2, 3, 4} && c != Vector4.{0, 2, 3, 4});
    assert(Quaternion.{} == Quaternion.{} && Quaternion.{0, 0, 0, 2} != Quaternion.{});
    m := Matrix4_Identity;
    n := m;
    assert(m == n);
    n._43 = 1;
    assert(m != n);
    assert(Matrix3_Identity == Matrix3_Identity && Matrix2_Identity == Matrix2_Identity);
    print("ok\n");
}
