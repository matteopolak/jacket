C :: #import "Compiler";
#run C.add_global_data(u8.[1], .USER_SEGMENT, cast(*C.Data_Segment) 42);
main :: () {}
