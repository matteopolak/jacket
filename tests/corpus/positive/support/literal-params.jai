#module_parameters (MODE: Usage_Mode = .WRITE, NAMES: [] string = .[]) ();
Usage_Mode :: enum { READ; WRITE; }
is_read :: () -> bool { return MODE == .READ; }
total :: () -> s64 { n := 0; for NAMES n += it.count; return n; }
