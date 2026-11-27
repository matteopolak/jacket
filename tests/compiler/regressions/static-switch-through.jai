// `#if value == { case ...; }` compiles the matching case and each case a `#through;` runs into,
// in a body and at file scope; the default `case;` is taken only when no value matches, wherever
// it is written.
#import "Basic";

N :: 2;

#if N == {
    case; KIND :: "default";
    case 2; #through;
    case 3; FROM_THROUGH :: true;
}

#if N == {
    case 1; OTHER :: "one";
    case; OTHER :: "default";
}

main :: () {
    seen := 0;

    #if N == {
        case 1; seen += 1;
        case 2;
            seen += 10;
            #through;
        case 3; seen += 100;
        case 4; seen += 1000;
    }

    assert(seen == 110);
    assert(FROM_THROUGH);
    assert(OTHER == "default");
    print("ok\n");
}
