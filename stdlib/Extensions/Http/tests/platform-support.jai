// test: skip_on(.WASM, reason = "Http server tests require native Io fibers")
#load "support/server_support.jai";

main :: () {
    assert(supported(), "native Http tests must not silently skip an implemented Io backend");
    finish();
}
