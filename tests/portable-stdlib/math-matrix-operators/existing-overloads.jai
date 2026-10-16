#import "Basic";
#import "Math";

main :: () {
    a := Vector2.{ 2, 3 };
    b := Vector2.{ 5, 7 };
    product := a * b;

    assert(product.x == 10 && product.y == 21);
    right := a * 2.0;
    left := 2.0 * a;

    assert(right.x == 4 && right.y == 6);
    assert(left.x == right.x && left.y == right.y);
    named := multiply(Matrix2_Identity, a);

    assert(named.x == a.x && named.y == a.y);
    matrix := multiply(Matrix2_Identity, Matrix2.{ _11 = 2, _12 = 3, _21 = 4, _22 = 5 });

    assert(matrix._11 == 2 && matrix._12 == 3 && matrix._21 == 4 && matrix._22 == 5);
    print("ok\n");
}
