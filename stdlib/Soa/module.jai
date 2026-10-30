// Soa: fixed-size arrays of structs stored as one array per member.

// `Soa(T, N)`: `N` values of the struct `T`, stored as one array per member ("structure of arrays") instead of one array of structs.
//
// The layout helps when a loop touches one member of many elements. Elements are read as whole `T` values
// with `soa[i]` (a copy) and written with `soa[i] = value`; an index outside `0..N-1` fails an assert. The
// extent is fixed at compile time, the storage lives in the value itself (no allocation) and starts zeroed.
// Constant, imported and overlapping members of `T` are not stored: reading gives them their default value.
//
// ```jai
// Particle :: struct { x: float; y: float; }
//
// particles: Soa(Particle, 3);
// particles[1] = .{ 2, 5 };
// assert(particles[1].y == 5 && particles[0].x == 0);
//
// particles.x[2] = 9; // a member is an ordinary array
// assert(particles[2].x == 9);
// ```
Soa :: Impl.Soa;
// Lets a `Soa` be used in a `for` loop over its `N` elements; `it` is a copy of the element and `it_index` its index.
//
// With `for *`, `it` points to a temporary copy that is stored back into the array after each pass of the body,
// so changes through it stick, but it is not a pointer into the arrays and must not be kept. Reverse
// iteration is not supported.
//
// ```jai
// Pair :: struct { a: int; b: int; }
//
// values: Soa(Pair, 2);
// for *pair, index: values pair.a = index + 10;
// total := 0;
// for pair: values total += pair.a;
// assert(total == 21);
// ```
for_expansion :: Impl.for_expansion;
// A small demonstration of writing and reading an element. It exists as a smoke test and is not part of the API.
example :: Impl.example;

using,only(.["[]", "[]="]) #import "internal/collections/soa";

#scope_file
Impl :: #import "internal/collections/soa";
