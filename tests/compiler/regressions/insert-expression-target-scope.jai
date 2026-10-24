// `#insert,scope(code)` in expression and type position resolves names where `code` was
// written: here a struct local to the caller's block, which the macro cannot see.
#import "Basic";

Wrap :: struct(T: Type) {
    v: T;
}

type_name :: (name: string) -> string {
    return name;
}

make :: ($name: string, $call := #caller_code) -> Wrap(#insert,scope(call) #run type_name(name)) {
    w: Wrap(#insert,scope(call) #run type_name(name));
    w.v.id = #insert,scope(call) "base";
    return w;
}

main :: () {
    {
        Local :: struct {
            id: int;
        }

        base :: 42;

        w := make("Local");
        l: Local = w.v;

        assert(l.id == 42);
    }

    print("ok\n");
}
