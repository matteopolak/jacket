// A run inside an `#if` branch only the target program takes (the host program has the host `OS`
// and `CPU`) names an import declared in that branch. The host program's copy of the file has no
// such name, so the run stays on the target compiler.
#if OS != BUILD_OS || CPU != BUILD_CPU {
    Q :: #import "Basic";

    #run {
        text := Q.tprint("%", 5);
        Q.assert(text == "5");
    }
}

main :: () {}
