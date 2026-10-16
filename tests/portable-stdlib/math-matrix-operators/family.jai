#import "Basic";
#import "Math";

coefficient :: (m: $T, row: int, col: int) -> float {
    if row < m.coef.count && col < m.coef[0].count return m.coef[row][col];
    if row == col return 1;
    return 0;
}

check_product :: (a: $A, b: $B, actual: $R) {
    inner := 4;

    #if A == Matrix2 inner = 2;
    #if A == Matrix3 && B == Matrix3 inner = 3;

    for i: 0..actual.coef.count - 1 {
        for j: 0..actual.coef[0].count - 1 {
            expected: float;
            for k: 0..inner - 1 expected += coefficient(a, i, k) * coefficient(b, k, j);
            assert(actual.coef[i][j] == expected);
        }
    }
}

check_components :: (a: $T/interface AnyMatrix) {
    scaled := a * 2.0;
    reversed := 2.0 * a;
    sum := a + a;
    difference := a - scaled;

    assert(type_of(scaled) == T && type_of(reversed) == T);
    assert(type_of(sum) == T && type_of(difference) == T);

    for i: 0..a.floats.count - 1 {
        assert(scaled.floats[i] == 2 * a.floats[i]);
        assert(reversed.floats[i] == scaled.floats[i]);
        assert(sum.floats[i] == scaled.floats[i]);
        assert(difference.floats[i] == -a.floats[i]);
    }
}

main :: () {
    a2 := Matrix2.{ 2, 3, 4, 5 };
    b2 := Matrix2.{ 6, 7, 8, 9 };
    a3 := Matrix3.{ 1, 2, 3, 4, 5, 6, 7, 8, 10 };
    b3 := Matrix3.{ 2, 0, 1, 3, 4, 5, 6, 7, 9 };
    a4 := Matrix4.{ 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 17 };
    b4 := Matrix4.{ 2, 0, 1, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 16 };
    aa := Matrix4x3.{ 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12 };
    ba := Matrix4x3.{ 2, 0, 1, 3, 4, 5, 6, 7, 8, 9, 10, 11 };

    check_product(a2, b2, a2 * b2);
    check_product(a3, b3, a3 * b3);
    check_product(a4, b3, a4 * b3);
    check_product(a4, b4, a4 * b4);
    check_product(aa, ba, aa * ba);
    check_product(a4, ba, a4 * ba);
    check_product(aa, b4, aa * b4);
    forward := a2 * b2;
    backward := b2 * a2;

    assert(forward._11 == 36 && backward._11 == 40);
    assert(type_of(a4 * ba) == Matrix4 && type_of(aa * b4) == Matrix4);
    assert(type_of(aa * ba) == Matrix4x3);

    v2 := a2 * Vector2.{ 3, 5 };
    v3 := a3 * Vector3.{ 2, 3, 4 };
    v4 := a4 * Vector4.{ 2, 3, 4, 5 };
    va := aa * Vector4.{ 2, 3, 4, 5 };

    assert(v2.x == 21 && v2.y == 37);
    assert(v3.x == 20 && v3.y == 47 && v3.z == 78);
    assert(v4.x == 40 && v4.y == 96 && v4.z == 152 && v4.w == 213);
    assert(va.x == 40 && va.y == 96 && va.z == 152);
    assert(type_of(va) == Vector3);
    check_components(a2);
    check_components(a3);
    check_components(a4);
    check_components(aa);
    assert(a2._11 == 2 && aa._14 == 4 && a4._44 == 17);
    print("ok\n");
}
