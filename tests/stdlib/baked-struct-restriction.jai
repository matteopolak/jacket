// `$V/Vec3` where `Vec3 :: #bake_arguments Vector(N = 3)` accepts `Vector(T, 3)` only.
#import "Basic";

Vector :: struct(T: Type, N: int) {
    data: [N] T;
}

Vec2 :: #bake_arguments Vector(N = 2);
Vec3 :: #bake_arguments Vector(N = 3);

which :: (v: $V/Vec2) -> int {
    return 2;
}

which :: (v: $V/Vec3) -> int {
    return 3;
}

main :: () {
    a: Vector(float32, 3);
    b: Vector(int, 2);
    if which(a) != 3 exit(1);
    if which(b) != 2 exit(2);
    print("ok\n");
}
