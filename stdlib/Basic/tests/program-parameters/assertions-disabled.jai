Basic :: #import "Basic"()(ENABLE_ASSERT = false);
Again :: #import "Basic";

#assert size_of(Basic.String_Builder.Buffer) == 24;
#assert Basic.Apollo_Time == Again.Apollo_Time;

main :: () {}
