// `#if` takes any constant that has a truth value: a float is true when nonzero, and `Code` is
// true unless it is the empty `#code,null`.
#import "Basic";

on_float :: ($w: float) -> string {
    #if w return "nonzero";
    else return "zero";
}

Element :: ($children := #code,null) #expand {
    #if children {
        #insert children;
    } else {
        `empty += 1;
    }
}

main :: () {
    assert(on_float(1.0) == "nonzero");
    assert(on_float(0.5) == "nonzero");
    assert(on_float(0.0) == "zero");

    empty, full := 0, 0;

    Element();
    Element(#code {
        full += 1;
    });
    Element();
    assert(empty == 2 && full == 1, "% %", empty, full);
    print("ok\n");
}
