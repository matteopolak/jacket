// A top-level `#insert #run` runs while the host twin loads the program. The error it reports is
// the program's error: the target compiler must not run the same code again.
#import "Basic";
#import "Compiler";

generate :: () -> string {
    print("generated on %\n", OS);
    compiler_report("the generator rejects this program");
    return "";
}

#insert #run generate();

main :: () {}
