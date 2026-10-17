M :: #import "Machine_X64";

#assert #run verify();

verify :: () -> bool {
    for 0..255 M.pause();
    return true;
}

main :: () {}
