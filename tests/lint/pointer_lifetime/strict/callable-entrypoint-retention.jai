#import "Basic";

saved: *int;

main :: () {
    local := 42;
    saved = *local;
}

caller :: () {
    main();
    print("%", saved.*);
}
