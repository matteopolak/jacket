#placeholder Generated;
answer :: (value: Generated = .{}) -> int { return value.answer; }
add_build_string :: (data: string, workspace: s64) #compiler;
#run add_build_string("Generated :: struct { answer: int = 42; }", -1);
main :: () -> int { return answer(); }
