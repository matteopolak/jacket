U :: union { a: int; b: float; c: [0] u8; }
main :: () { u: U; u.a = 1; f := u.b; }
