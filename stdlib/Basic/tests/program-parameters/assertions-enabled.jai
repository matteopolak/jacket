Basic :: #import "Basic"()(ENABLE_ASSERT = true);
Again :: #import "Basic";

#assert size_of(Basic.String_Builder.Buffer) == 3 * size_of(s64) + size_of(*void);
#assert Basic.Apollo_Time == Again.Apollo_Time;

main :: () {}
