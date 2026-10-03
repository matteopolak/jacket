capture_name :: (destination: *string) #expand {
    destination.* = #procedure_name(#this);
}

worker :: () -> int {
    name: string;
    capture_name(*name);
    if name != "worker" return 1;
    return 42;
}

main :: () -> int {
    name: string;
    capture_name(*name);
    if name != "main" return 2;
    return worker();
}
