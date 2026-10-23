// `#code` can hold a `return` statement that ends at the closing `)` of the call.
m :: (c: Code) #expand {
    #insert c;
}

f :: () -> int {
    m(#code return 5);
    return 1;
}

main :: () {
    f();
}
