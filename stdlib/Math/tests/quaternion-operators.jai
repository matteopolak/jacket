// Quaternions add, subtract, negate and scale by components; `*` is the Hamilton product.
#import "Basic";
#import "Math";

main :: () {
    a := Quaternion.{ 1, 2, 3, 4 };
    b := Quaternion.{ 5, 6, 7, 8 };

    assert(a + b == Quaternion.{ 6, 8, 10, 12 });
    assert(b - a == Quaternion.{ 4, 4, 4, 4 });
    assert(-a == Quaternion.{ -1, -2, -3, -4 });
    assert(a * 2 == Quaternion.{ 2, 4, 6, 8 } && 0.5 * b == Quaternion.{ 2.5, 3, 3.5, 4 });

    // (1 + 2i + 3j + 4k)(5 + 6i + 7j + 8k) with the scalar part in `w`.
    assert(a * b == Quaternion.{ 24, 48, 48, -6 });
    assert(b * a == Quaternion.{ 32, 32, 56, -6 });

    i := Quaternion.{ 1, 0, 0, 0 };
    j := Quaternion.{ 0, 1, 0, 0 };
    k := Quaternion.{ 0, 0, 1, 0 };
    minus_one := Quaternion.{ 0, 0, 0, -1 };

    assert(i * j == k && j * k == i && k * i == j);
    assert(j * i == -k);
    assert(i * i == minus_one && j * j == minus_one && k * k == minus_one);

    one := Quaternion.{ 0, 0, 0, 1 };

    assert(a * one == a && one * a == a);

    quarter_turn: Quaternion;

    set_from_axis_and_angle(*quarter_turn, .{ 0, 0, 1 }, PI / 2);
    r := quarter_turn * quarter_turn;

    assert(abs(r.z - 1) < 0.0001 && abs(r.w) < 0.0001);

    q := a;
    q += b;
    q -= a;
    assert(q == b);
    print("ok\n");
}
