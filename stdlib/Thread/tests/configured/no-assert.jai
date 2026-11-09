// test: skip_on(.WASM, reason = "needs the Thread module, which has no wasm target (no OS threads)")
// Thread built against a Basic whose assertions are compiled out: only the main program may
// set program parameters, and every importer of Basic shares them.
#import "Basic"()(ENABLE_ASSERT = false);
T :: #import "Thread";

#assert !#exists(String_Builder.Buffer.ensured_count);

main :: () {
    semaphore: T.Semaphore;

    T.init(*semaphore);
    T.signal(*semaphore);
    print("%\n", T.wait_for(*semaphore, 0) == .SUCCESS);
    T.destroy(*semaphore);
}
