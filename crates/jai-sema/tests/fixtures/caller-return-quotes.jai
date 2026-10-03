wrap :: (body: Code) #expand { #insert body; }
finish :: () #expand { `return 42; }
answer :: () -> int {
    code :: #code { finish(); };
    wrap(code);
}
main :: () -> int { return answer(); }
