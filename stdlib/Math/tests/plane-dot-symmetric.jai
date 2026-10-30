// `plane_dot` is `#symmetric`: the plane and the point can be given in either order.
#import "Basic";
#import "Math";

main :: () {
    p := make_plane(xyz(0, 1, 0), xyz(0, 2, 0));

    assert(plane_dot(p, xyz(0, 5, 0)) == 3);
    assert(plane_dot(xyz(0, 5, 0), p) == 3);
    assert(plane_dot(xyz(0, 0, 0), p) == -2);
    print("ok\n");
}
