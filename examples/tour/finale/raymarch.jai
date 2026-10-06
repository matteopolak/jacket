// Finale: a ray marcher that draws a 3D scene with text characters.
//
// Each character cell shoots a ray into a scene described by a signed distance
// function: for any point, how far away is the nearest surface? The ray steps
// forward by that distance until it touches something, then the surface is
// shaded by how directly it faces the light, with a shadow ray for good measure.
// Try moving the light, the camera, or the blobs!

WIDTH :: 60;
HEIGHT :: 26;
SHADES :: " .,:;-=+*#%@";

CAMERA :: Vector3.{ 0.5, 1.6, -3.2 };
LIGHT_DIRECTION :: #run normalize(Vector3.{ -0.7, 1, -0.6 });

tour_finale :: () {
    // Rays aim at a point slightly above the floor; characters are about twice as tall as wide.
    forward := normalize(Vector3.{ 0.1, 0.7, 0 } - CAMERA);
    right := normalize(cross(Vector3.{ 0, 1, 0 }, forward));
    up := cross(forward, right);

    line: [WIDTH] u8;
    for row: 0..HEIGHT - 1 {
        for column: 0..WIDTH - 1 {
            u := (cast(float) column / (WIDTH - 1) - 0.5) * 1.45;
            v := (0.5 - cast(float) row / (HEIGHT - 1)) * 1.25;
            ray := normalize(forward + right * u + up * v);
            index := shade(CAMERA, ray);
            // Sprinkle a few stars over the top of the sky.
            scattered := (column * 7 + row * 13) % 23 == 0 && (column * row) % 5 == 1;
            star := row < HEIGHT / 5 && scattered;
            if index == 0 && star {
                line[column] = #char "*";
            } else {
                line[column] = SHADES[index];
            }
        }
        print("%\n", cast(string) line);
    }
}

// Returns an index into SHADES for the ray from `origin` in `direction`.
shade :: (origin: Vector3, direction: Vector3) -> int {
    distance := 0.0;
    for 1..80 {
        point := origin + direction * distance;
        nearest, material := scene(point);
        if nearest < 0.002 {
            normal := surface_normal(point);
            light := max(dot(normal, LIGHT_DIRECTION), 0);
            if in_shadow(point + normal * 0.01) light *= 0.2;
            if material == .BLOB {
                // A little ambient light keeps the dark side from vanishing into the floor.
                light = 0.25 + 0.75 * light;
            } else {
                // A checkerboard: the floor alternates between two brightnesses.
                checker := (cast(int) floor(point.x) + cast(int) floor(point.z)) & 1;
                light *= ifx checker then 0.35 else 0.7;
                // Fog: the floor fades out towards the horizon.
                light *= exp(-0.12 * distance);
            }
            return 1 + cast(int) (light * (SHADES.count - 2));
        }
        distance += nearest;
        if distance > 30 break;
    }
    return 0; // The ray flew off into the sky.
}

Material :: enum {
    FLOOR;
    BLOB;
}

// The scene: two spheres melted together with a smooth minimum, above a floor.
scene :: (p: Vector3) -> float, Material {
    big := length(p - Vector3.{ -0.4, 1, 0.3 }) - 0.9;
    small := length(p - Vector3.{ 0.9, 0.55, -0.4 }) - 0.5;
    blob := smooth_min(big, small, 0.4);
    floor_distance := p.y;
    if floor_distance < blob return floor_distance, .FLOOR;
    return blob, .BLOB;
}

smooth_min :: (a: float, b: float, k: float) -> float {
    h := clamp(0.5 + 0.5 * (b - a) / k, 0, 1);
    return lerp(b, a, h) - k * h * (1 - h);
}

// The gradient of the distance field points away from the surface.
surface_normal :: (p: Vector3) -> Vector3 {
    e :: 0.001;
    return normalize(Vector3.{
        scene(p + Vector3.{ e, 0, 0 }) - scene(p - Vector3.{ e, 0, 0 }),
        scene(p + Vector3.{ 0, e, 0 }) - scene(p - Vector3.{ 0, e, 0 }),
        scene(p + Vector3.{ 0, 0, e }) - scene(p - Vector3.{ 0, 0, e }),
    });
}

in_shadow :: (p: Vector3) -> bool {
    travelled := 0.0;
    for 1..40 {
        nearest := scene(p + LIGHT_DIRECTION * travelled);
        if nearest < 0.002 return true;
        travelled += nearest;
        if travelled > 10 break;
    }
    return false;
}
