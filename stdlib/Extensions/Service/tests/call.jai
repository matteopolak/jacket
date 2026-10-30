// test: skip_on(.WASM, reason = "compile-time services exist only inside the compiler")
// Compile-time calls to a service (services/upper.jai), as a build makes them.
//
//     jaic stdlib/Extensions/Service/tests/call.jai -interp

#import "Basic";
Service :: #import "Extensions/Service";

UPPER :: #run tprint("%services/upper.jai", #filepath);

ask :: (request: string) -> string {
    status, reply := Service.call(UPPER, request);
    return tprint("% %", status, reply);
}

GREETING :: #run ask("hello");
AGAIN :: #run ask("again"); // the same worker answers
FAILED :: #run ask("fail:no table");
GONE :: #run ask("gone:no database");

MISSING :: #run -> string {
    status := Service.call(tprint("%missing.jai", #filepath), "x");
    return tprint("%", status);
};

main :: () {
    print("%\n%\n%\n%\n%\n", GREETING, AGAIN, FAILED, GONE, MISSING);
    // Not at compile time: no service is asked.
    status := Service.call(UPPER, "runtime");
    print("%\n", status);
}
