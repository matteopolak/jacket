// `#code` takes a dotted path with a string-literal member.
f :: (c: Code) #expand {}

main :: () {
    f(#code a.b."c");
}
