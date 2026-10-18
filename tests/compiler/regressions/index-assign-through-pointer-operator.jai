// An indexed assignment target on a type with `operator *[]` writes through the returned
// pointer, plain and compound; reads of the same expression keep using `operator []`.
#import "Basic";

B :: struct { a: [2] int; }

gets := 0;
refs := 0;

operator [] :: (b: B, i: int) -> int { gets += 1; return b.a[i]; }
operator *[] :: (b: *B, i: int) -> *int { refs += 1; return *b.a[i]; }

Uniform :: struct { name: int; ownership: bool; }
Uniforms :: struct { fixed: [4] Uniform; fixed_count: int; dynamic: [..] Uniform; }

operator [] :: (uniforms: Uniforms, index: int) -> Uniform {
    if uniforms.dynamic.data { return uniforms.dynamic[index]; } else { return uniforms.fixed[index]; }
}
operator *[] :: (uniforms: *Uniforms, index: int) -> *Uniform {
    if uniforms.dynamic.data { return *uniforms.dynamic[index]; } else { return *uniforms.fixed[index]; }
}

set :: (uniforms: *Uniforms, created: Uniform) -> bool {
    was_owned := false;
    for i: 0..uniforms.fixed_count - 1 {
        if uniforms.*[i].name == created.name {
            was_owned = uniforms.*[i].ownership;
            uniforms.*[i] = created;
            return was_owned;
        }
    }
    return false;
}

main :: () {
    b: B;
    p := *b;
    p.*[1] = 5;
    b[0] = 7;
    if refs != 2 || gets != 0 exit(1);
    x := b[1];
    y := p.*[0];
    if gets != 2 || refs != 2 exit(2);
    if x != 5 || y != 7 || b.a[0] != 7 || b.a[1] != 5 exit(3);

    b[0] += 3;
    b[1] -= 1;
    if b.a[0] != 10 || b.a[1] != 4 exit(4);

    u: Uniforms;
    u.fixed_count = 1;
    u.fixed[0] = .{name = 1, ownership = true};
    if !set(*u, .{name = 1, ownership = false}) exit(5);
    if u.fixed[0].ownership exit(6);
}
