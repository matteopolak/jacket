make_declarations :: () -> Code {
    return #code ANSWER :: 42;
}
#insert #run make_declarations();
main :: () -> int { return ANSWER; }
