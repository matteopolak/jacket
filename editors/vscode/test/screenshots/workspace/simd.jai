#import "Basic";

// Adds eight pairs of s32 lanes with one AVX2 instruction.
add_lanes :: (a: [8] s32, b: [8] s32) -> [8] s32 {
    sum: [8] s32;
    pa, pb, psum := a.data, b.data, sum.data;
    #asm AVX2 {
        movdqu.y va:, [pa];
        movdqu.y vb:, [pb];
        vpaddd.y va, va, vb;
        movdqu.y [psum], va;
    }
    return sum;
}

main :: () {
    a := s32.[1, 2, 3, 4, 5, 6, 7, 8];
    b := s32.[10, 20, 30, 40, 50, 60, 70, 80];
    print("%\n", add_lanes(a, b));
}
