Basic :: #import "Basic"()(ENABLE_ASSERT = false);
Again :: #import "Basic";

#assert size_of(Basic.String_Builder.Buffer) == ((2 * size_of(s64) + size_of(*void)) + align_of(Basic.String_Builder.Buffer) - 1) / align_of(Basic.String_Builder.Buffer) * align_of(Basic.String_Builder.Buffer);
#assert Basic.Apollo_Time == Again.Apollo_Time;

main :: () {}
