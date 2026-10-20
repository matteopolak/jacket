// Floating-point loops over an array of structs. Prints the final energy.
#import "Basic";

Body :: struct {
    x, y, vx, vy: float64;
    mass: float64;
}

main :: () {
    bodies: [16] Body;
    for *bodies {
        it.x = cast(float64) it_index;
        it.y = cast(float64) (it_index * it_index % 7);
        it.mass = 1.0 + cast(float64) (it_index % 3);
    }

    for 1..60_000 {
        for *a: bodies {
            for b: bodies {
                dx := b.x - a.x;
                dy := b.y - a.y;
                distance := dx * dx + dy * dy + 0.01;

                a.vx += dx * b.mass / (distance * 100.0);
                a.vy += dy * b.mass / (distance * 100.0);
            }
        }

        for *bodies {
            it.x += it.vx;
            it.y += it.vy;
        }
    }

    energy: float64;
    for bodies energy += 0.5 * it.mass * (it.vx * it.vx + it.vy * it.vy);

    print("%\n", cast(int) (energy * 1000.0));
}
