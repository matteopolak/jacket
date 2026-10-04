Flags :: enum_flags u8 { A; B; }
main :: () -> int { v: Flags = .B; return cast(int) v.A + cast(int) v.B; }
