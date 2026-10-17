// Extensions/Io sleeps, deadlines and cancellation over real IO: read, accept and write that time
// out or are canceled, nested deadlines, and a scheduler that ends with fibers still blocked.
#load "support/suite.jai";

main :: () {
    run_group(run_timeouts);
    print("ok (backend %)\n", last_backend);
}
