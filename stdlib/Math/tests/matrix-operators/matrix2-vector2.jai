#import "Basic";
#import "Math";

main :: () {
    identity_result := Matrix2_Identity * Vector2.{ 3, 5 };

    assert(identity_result.x == 3 && identity_result.y == 5);

    m := Matrix2.{ _11 = 2, _12 = 3, _21 = -4, _22 = 5 };
    v := Vector2.{ 7, -2 };
    result := m * v;

    assert(result.x == 8 && result.y == -38);
    named := multiply(m, v);

    assert(result.x == named.x && result.y == named.y);
    assert(m._11 == 2 && m._12 == 3 && m._21 == -4 && m._22 == 5);
    assert(v.x == 7 && v.y == -2);
    assert(type_of(result) == Vector2);

    zero := m * Vector2.{};

    assert(zero.x == 0 && zero.y == 0);
    rotated := Matrix2.{ _11 = 0, _12 = -1, _21 = 1, _22 = 0 } * Vector2.{ 3, 5 };

    assert(rotated.x == -5 && rotated.y == 3);
    chained := m * (Matrix2_Identity * v);

    assert(chained.x == 8 && chained.y == -38);
    print("ok\n");
}
