// Provenance: reviewed Example_Plugin public record and lifecycle contract.
using #import "Basic";
C :: #import "Compiler";
E :: #import "Example_Plugin";

main :: () {
    record: E.My_Plugin;

    assert(record.some_state == 0);
    assert((cast(*Type_Info_Struct) type_info(E.My_Plugin)).name == "My_Plugin");
    p := E.get_plugin();

    assert(p != null);
    assert(p.init && p.before_intercept && p.add_source && p.message && p.finish && p.shutdown && p.log_help);
    assert(p.init(p, .["-option", "one", "-option", "two"]));
    assert((cast(*E.My_Plugin) p).some_state == 33);
    flags: C.Intercept_Flags;

    p.before_intercept(p, *flags);
    assert(flags == 0);
    p.add_source(p);
    phase: C.Message_Phase;
    phase.kind = .PHASE;
    phase.phase = .PRE_WRITE_EXECUTABLE;
    phase.executable_name = "test-program";
    p.message(p, *phase);
    typed: C.Message_Typechecked;
    typed.kind = .TYPECHECKED;
    p.message(p, *typed);
    complete: C.Message_Complete;
    complete.kind = .COMPLETE;
    p.message(p, *complete);
    p.finish(p);
    p.log_help(p);
    p.shutdown(p);

    for options: .[cast([] string) .["-option"], .["unknown"], .["-option", "ok", "unknown"]] {
        failed := E.get_plugin();
        assert(!failed.init(failed, options));
        failed.shutdown(failed);
    }

    empty := E.get_plugin();

    assert(empty.init(empty, .[]));
    assert((cast(*E.My_Plugin) empty).some_state == 33);
    empty.shutdown(empty);
    print("ok\n");
}
