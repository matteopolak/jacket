E :: enum { A; B; }
F :: enum_flags { X; Y; }
main :: () { e := E.A + E.B; f := F.X | 5; g := ~F.X; h := cast(E) -1; i := E.A < 5; }
