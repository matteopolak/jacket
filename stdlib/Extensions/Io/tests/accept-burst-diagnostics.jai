// Same burst count/deadline/assertions as the full suite. Backend selection is
// the caller's existing JAI_IO_BACKEND; no fallback or retry is introduced here.
#load "support/suite.jai";

main :: () {
    run_group(test_accept_bursts);
    print("accept burst passed (%)\n", last_backend);
}
