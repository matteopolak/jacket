#import "Basic";
Math :: #import "Math";

Vector4 :: struct {
    x, y, z, w: float;
}

dot_product :: (a: Vector4, b: Vector4) -> float {
    return a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w;
}

#poke_name Math dot_product;

main :: () {
    assert(type_info(Vector4) != type_info(Math.Vector4));
    assert(type_info(Vector4).name == "Vector4" && type_info(Math.Vector4).name == "Vector4");
    a := Vector4.{ 1, 2, 3, 4 };
    b := Vector4.{ 6, 7, 8, 9 };

    assert(Math.dot(a, b) == 80 && Math.dot_product(a, b) == 80);
    assert(Math.dot(Math.Vector2.{ 1, 2 }, Math.Vector2.{ 3, 4 }) == 11);
    assert(Math.dot(Math.Vector3.{ 1, 2, 3 }, Math.Vector3.{ 4, 5, 6 }) == 32);
    assert(Math.dot(Math.Vector4.{ 1, 2, 3, 4 }, Math.Vector4.{ 6, 7, 8, 9 }) == 80);
    assert(Math.dot(Math.Quaternion.{ 1, 2, 3, 4 }, Math.Quaternion.{ 6, 7, 8, 9 }) == 80);
    alias: (Math.Vector4, Math.Vector4) -> float = Math.dot;
    canonical: (Math.Vector4, Math.Vector4) -> float = Math.dot_product;

    assert(alias == canonical);
    print("ok\n");
}
