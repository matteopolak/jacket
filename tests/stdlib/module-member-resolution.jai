// Every way of reaching a module's members resolves through `module_lookup`, so a name visible
// through a plain `#import` must also be visible through a named import, `using` of that import,
// and a re-exporting module. Exported `using` globals (GL's procedure table) were once missed
// by the named-import path.
#import "Basic";
#import "Using_Global";
UG :: #import "Using_Global";
RO :: #import "Reexport_Outer";
PP :: #import "Program_Param"()(VERBOSE = true);


main :: () {
    load_procs();
    // Exported `using` global, reached four ways.
    assert(answer() == 42);
    assert(UG.answer() == 42);
    assert(UG.procs.answer() == 42);
    {
        using UG;
        assert(answer() == 42);
    }

    // Declarations re-exported with `using M :: #import`.
    t := RO.Inner_Thing.{ number = 1 };
    assert(t.number + RO.inner_value() == 8);

    // Module parameters.
    assert(PP.VERBOSE);
    assert(PP.is_verbose());
    print("ok\n");
}
