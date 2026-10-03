#placeholder Generated;
value: Generated;
add_build_string :: (data: string, workspace: s64) #compiler;
#run add_build_string("Generated :: struct { answer: int = 42; }", -1);
main :: () -> int { return value.answer; }
