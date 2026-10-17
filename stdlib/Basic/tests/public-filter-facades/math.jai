#import "Basic";
#import "Math";
M :: #import "Math";
Again :: #import "Math";

main :: () {
    result := Matrix2_Identity * Vector2.{ 3, 5 };

    assert(result.x == 3 && result.y == 5);
    a := Vector2.{ 8, 6 };
    sum := a + Vector2.{ 2, 3 };
    difference := a - Vector2.{ 2, 3 };
    divided := a / 2.0;
    scaled := 2.0 * a;

    assert(sum.x == 10 && sum.y == 9 && difference.x == 6 && difference.y == 3);
    assert(divided.x == 4 && divided.y == 3 && scaled.x == 16 && scaled.y == 12);
    first: (Matrix2, Vector2) -> Vector2 = M.operator *;
    second: (Again.Matrix2, Again.Vector2) -> Again.Vector2 = Again.operator *;

    assert(first == second && type_info(M.Matrix2) == type_info(Again.Matrix2));
    print("ok\n");
}
