#placeholder ANSWER;
add_build_string :: (data: string, workspace: s64) #compiler;
#run add_build_string("ANSWER :: 42;", -1);
main :: () -> int { return ANSWER; }
