Basic :: #import "Basic"()(ENABLE_ASSERT = true);
Again :: #import "Basic";

#assert size_of(Basic.String_Builder.Buffer) == 32;
#assert Basic.Apollo_Time == Again.Apollo_Time;

main :: () {}
