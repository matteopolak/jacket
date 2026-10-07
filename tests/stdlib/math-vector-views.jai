// Math vectors expose overlaid views (xy, xyz, component...); Quaternion.{} is the identity.
#import "Basic";
#import "Math";

main :: () {
    v := Vector4.{ 1, 2, 3, 4 };
    assert(v.xyz.z == 3 && v.zw.y == 4 && v.component[1] == 2 && v.yzw.x == 2);
    v.xy = .{ 9, 8 };
    assert(v.x == 9 && v.y == 8 && size_of(Vector4) == 16);
    q: Quaternion;
    assert(q.w == 1 && q.xyz.x == 0 && size_of(Quaternion) == 16);
    u := Vector3.{ 1, 2, 3 };
    u.component[2] = 7;
    assert(u.z == 7 && u.yz.x == 2);
}
