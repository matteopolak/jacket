// A file imported while compile-time code is already running can declare `#add_context`. The
// members land after the existing ones, take their default values and keep working through the
// context the running code started with.
#import "Basic";

use_late :: () {
    first := context.thread_index;
    Late :: #import,file "modules/late_context/module.jai";

    assert(first == 0);
    assert(Late.read_late() == 31);
    assert(Late.text_late() == "late");
    Late.write_late(5);
    assert(Late.read_late() == 5);
    print("ok\n");
}

#run use_late();

main :: () {}
