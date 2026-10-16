#import "Basic";
#import "Math";

main :: () {
    result := Matrix2_Identity * Vector2.{ 3, 5 };
    assert(result.x == 3 && result.y == 5);
}
